#![windows_subsystem = "windows"]

mod chart;
mod pdh;
mod perf;
mod pid;
mod renderer;
mod window;
mod windows_utils;

use std::time::Duration;

use chart::ChartSurface;
use perf::PerfTracker;
use pid::{get_current_dwm_pid, parse_pid};
use renderer::Renderer;
use window::Window;
use windows::{
    core::{w, Result, HSTRING},
    Foundation::{Numerics::Vector2, TypedEventHandler},
    Win32::{
        Foundation::E_FAIL,
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
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
        get_current_dwm_pid()?
    };
    let perf_tracker = PerfTracker::new(process_id)?;

    let timer = queue.CreateTimer()?;
    timer.SetInterval(Duration::from_secs(1).into())?;
    timer.SetIsRepeating(true)?;
    let timer_token = timer.Tick(&TypedEventHandler::<_, _>::new({
        // SAFETY: We know that the timer will only tick on the same thread
        // as the dispatcher queue (our UI thread). As long as we remove the tick
        // handler before the end of the lifetime of our perf tracker object,
        // we should be fine.
        let perf_tracker: u64 = &perf_tracker as *const _ as _;
        move |_, _| -> Result<()> {
            let perf_tracker = unsafe { (perf_tracker as *const PerfTracker).as_ref().unwrap() };
            let utilization_value = perf_tracker.get_current_value()?;
            chart.add_point(utilization_value as f32);
            chart.redraw(&renderer)?;
            Ok(())
        }
    }))?;

    perf_tracker.start()?;
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
    // SAFETY: There isn't a race here with the tick handler because we are
    // no longer pumping messages.
    timer.RemoveTick(timer_token)?;
    timer.Stop()?;
    perf_tracker.close()?;
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
