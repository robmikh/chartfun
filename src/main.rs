#![windows_subsystem = "windows"]

mod app;
mod chart;
mod pdh;
mod perf;
mod pid;
mod renderer;
mod text_block;
mod window;
mod windows_utils;

use app::App;
use pid::{get_current_dwm_pid, parse_pid};
use window::Window;
use windows::{
    core::{w, Result, HSTRING},
    Win32::{
        Foundation::E_FAIL,
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        UI::{
            HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            WindowsAndMessaging::{
                DispatchMessageW, GetMessageW, MessageBoxW, TranslateMessage, MB_ICONERROR, MSG,
            },
        },
    },
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

    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
    }
    unsafe { RoInitialize(RO_INIT_SINGLETHREADED)? };
    let controller = create_dispatcher_queue_controller_for_current_thread()?;

    let window_width = 432;
    let window_height = 362;
    let mut window = Window::new("chartfun", window_width, window_height)?;
    let dpi = window.dpi();

    let process_id = if let Some(pid) = pid {
        pid
    } else {
        get_current_dwm_pid()?
    };

    let app = App::new(process_id, dpi)?;
    let root = app.root().clone();
    let compositor = app.compositor().clone();

    window.set_app(app);
    window.show();
    let target = compositor.create_desktop_window_target(window.handle(), false)?;
    target.SetRoot(&root)?;

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    window.shutdown_app()?;
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
