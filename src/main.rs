#![windows_subsystem = "windows"]

mod chart;
mod pdh;
mod perf;
mod pid;
mod renderer;
mod text_block;
mod window;
mod windows_utils;

use std::time::Duration;

use chart::ChartSurface;
use perf::PerfTracker;
use pid::{get_current_dwm_pid, get_name_from_pid, parse_pid};
use renderer::Renderer;
use text_block::TextBlock;
use window::Window;
use windows::{
    core::{w, Result, HSTRING},
    Foundation::{
        Numerics::{Vector2, Vector3},
        TypedEventHandler,
    },
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
    numerics::ToVector2,
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

    let window_width = 432;
    let window_height = 362;

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
    visual.SetSize(chart.size().to_vector2())?;
    visual.SetRelativeOffsetAdjustment(Vector3::new(0.5, 0.5, 0.0))?;
    visual.SetAnchorPoint(Vector2::new(0.5, 0.5))?;
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

    let process_name = get_name_from_pid(process_id)?;
    let process_name_text = TextBlock::new(
        &renderer,
        process_name,
        Color {
            A: 255,
            R: 0,
            G: 0,
            B: 0,
        },
    )?;

    let mut utilization_text = TextBlock::new(
        &renderer,
        "0%".to_owned(),
        Color {
            A: 255,
            R: 112,
            G: 112,
            B: 112,
        },
    )?;
    let utilization_text_root = utilization_text.root();
    utilization_text_root.SetAnchorPoint(Vector2::new(1.0, 0.0))?;
    utilization_text_root.SetRelativeOffsetAdjustment(Vector3::new(1.0, 0.0, 0.0))?;

    let info_height = {
        let process_name_height = process_name_text.root().Size()?;
        let utilization_height = utilization_text_root.Size()?;
        process_name_height.Y.max(utilization_height.Y)
    };

    let info_root = compositor.CreateContainerVisual()?;
    info_root.SetRelativeSizeAdjustment(Vector2::new(1.0, 0.0))?;
    info_root.SetSize(Vector2::new(0.0, info_height))?;
    info_root.SetOffset(Vector3::new(0.0, -info_height, 0.0))?;
    visual.Children()?.InsertAtTop(&info_root)?;

    let info_root_children = info_root.Children()?;
    info_root_children.InsertAtTop(process_name_text.root())?;
    info_root_children.InsertAtTop(utilization_text_root)?;

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
            utilization_text.set_text(&renderer, format!("{}%", utilization_value as i32))?;
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
