#![windows_subsystem = "windows"]

mod chart;
mod pdh;
mod pid;
mod renderer;
mod window;
mod windows_utils;

use std::time::Duration;

use chart::ChartSurface;
use pid::parse_pid;
use processdumper::{find_process_id_with_name_in_session, get_session_for_current_process};
use renderer::Renderer;
use window::Window;
use windows::{
    core::{w, Result, HSTRING},
    Foundation::{Numerics::Vector2, TypedEventHandler},
    Win32::{
        Foundation::E_FAIL,
        System::{
            Performance::{
                PdhCollectQueryData, PdhGetFormattedCounterValue, PDH_CSTATUS_VALID_DATA,
                PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
            },
            WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        },
        UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, MessageBoxW, TranslateMessage, MB_ICONERROR, MSG,
        },
    },
    UI::{Color, Composition::CompositionStretch},
};
use windows_utils::{
    composition::CompositionInterop,
    dispatcher_queue::{
        create_dispatcher_queue_controller_for_current_thread,
        shutdown_dispatcher_queue_controller_and_wait,
    },
};

use crate::pdh::{add_perf_counters, PerfQueryHandle, PDH_FUNCTION};

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let pid = if let Some(pid_string) = args.get(0) {
        if let Ok(pid) = parse_pid(&pid_string) {
            Some(pid)
        } else {
            return Err(windows::core::Error::new(
                E_FAIL,
                "Failed to parse process id!",
            ));
        }
    } else {
        None
    };

    unsafe { RoInitialize(RO_INIT_SINGLETHREADED)? };
    let controller = create_dispatcher_queue_controller_for_current_thread()?;
    let queue = controller.DispatcherQueue()?;

    let window_width = 800;
    let window_height = 600;

    let renderer = Renderer::new()?;

    let compositor = renderer.compositor.clone();
    let root = compositor.CreateSpriteVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    root.SetBrush(&compositor.CreateColorBrushWithColor(Color {
        A: 255,
        R: 255,
        G: 255,
        B: 255,
    })?)?;

    let mut chart = ChartSurface::new(&renderer)?;
    let visual = compositor.CreateSpriteVisual()?;
    visual.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    let brush = compositor.CreateSurfaceBrushWithSurface(chart.surface())?;
    brush.SetStretch(CompositionStretch::None)?;
    visual.SetBrush(&brush)?;
    root.Children()?.InsertAtTop(&visual)?;
    chart.redraw(&renderer)?;

    let process_id = if let Some(pid) = pid {
        pid
    } else {
        // During RDP sessions, you'll have multiple sessions and muiltple
        // DWMs. We want the one the user is currently using, so find the
        // session our program is running in.
        let current_session = get_session_for_current_process()?;
        let process_id = if let Some(process_id) =
            find_process_id_with_name_in_session("dwm.exe", current_session)?
        {
            process_id
        } else {
            return Err(windows::core::Error::new(
                E_FAIL,
                "Could not find a dwm process for this session!",
            ));
        };
        process_id
    };

    let counter_path = format!(
        r#"\GPU Engine(pid_{}*engtype_3D)\Utilization Percentage"#,
        process_id
    );

    let mut query_handle = PerfQueryHandle::open_query()?;
    let counter_handles = add_perf_counters(&query_handle, &counter_path)?;

    let timer = queue.CreateTimer()?;
    timer.SetInterval(Duration::from_secs(1).into())?;
    timer.SetIsRepeating(true)?;
    let timer_token = timer.Tick(&TypedEventHandler::<_, _>::new(move |_, _| -> Result<()> {
        unsafe {
            PDH_FUNCTION(PdhCollectQueryData(query_handle.0)).ok()?;
        }

        let mut utilization_value = 0.0;
        for counter_handle in &counter_handles {
            let counter_value = unsafe {
                let mut counter_type = 0;
                let mut counter_value = PDH_FMT_COUNTERVALUE::default();
                PDH_FUNCTION(PdhGetFormattedCounterValue(
                    *counter_handle,
                    PDH_FMT_DOUBLE,
                    Some(&mut counter_type),
                    &mut counter_value,
                ))
                .ok()?;
                counter_value
            };
            assert_eq!(counter_value.CStatus, PDH_CSTATUS_VALID_DATA);
            let value = unsafe { counter_value.Anonymous.doubleValue };
            utilization_value += value;
        }

        chart.add_point(utilization_value as f32);
        chart.redraw(&renderer)?;
        Ok(())
    }))?;

    unsafe {
        PDH_FUNCTION(PdhCollectQueryData(query_handle.0)).ok()?;
    }
    timer.Start()?;

    let window = Window::new("chartfun", window_width, window_height)?;
    let target = compositor.create_desktop_window_target(window.handle(), false)?;
    target.SetRoot(&root)?;

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    timer.RemoveTick(timer_token)?;
    query_handle.close_query()?;
    let _ = shutdown_dispatcher_queue_controller_and_wait(&controller, message.wParam.0 as i32)?;
    Ok(())
}

fn main() -> Result<()> {
    if let Err(error) = run() {
        let message = HSTRING::from(&format!("0x{:08X} - {}", error.code().0, error.message()));
        unsafe {
            let _ = MessageBoxW(None, &message, w!("chartfun"), MB_ICONERROR);
        };
        Err(error)
    } else {
        Ok(())
    }
}
