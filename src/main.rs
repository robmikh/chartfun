#![cfg_attr(not(feature = "verbose"), windows_subsystem = "windows")]

mod adapter;
mod app;
mod chart;
mod cli;
mod pdh;
mod perf;
mod pid;
mod renderer;
mod text_block;
mod window;
mod windows_utils;
mod sources;

use adapter::Adapter;
use app::App;
use cli::parse_args;
use pid::get_current_dwm_pid;
use window::Window;
use windows::{
    core::{w, Result, HSTRING},
    Win32::{
        Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1},
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        UI::{
            HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            WindowsAndMessaging::{
                DispatchMessageW, GetMessageW, MessageBoxW, TranslateMessage, MB_ICONERROR,
                MB_ICONINFORMATION, MSG,
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

use crate::sources::gpu_util::GpuUtilization;

fn run() -> Result<()> {
    let args = parse_args()?;
    let pid = args.pid;

    let dxgi_factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1()? };
    let adapter = if let Some(adapter_index) = args.adapter {
        let dxgi_adapter = unsafe { dxgi_factory.EnumAdapters1(adapter_index)? };
        let adapter = Adapter::from_dxgi_adapter(&dxgi_adapter)?;

        let message_string = format!("Using: {}", adapter.name);
        let message = HSTRING::from(&message_string);
        unsafe {
            let _ = MessageBoxW(None, &message, w!("chartfun"), MB_ICONINFORMATION);
        };
        if cfg!(feature = "verbose") {
            println!("{}", message_string);
        }

        Some(adapter.luid)
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

    let data_source = Box::new(GpuUtilization::new(process_id, adapter)?);
    let app = App::new(dpi, data_source)?;
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
