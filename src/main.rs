mod chart;
mod renderer;
mod window;
mod windows_utils;

use std::time::Duration;

use chart::ChartSurface;
use rand::Rng;
use renderer::Renderer;
use window::Window;
use windows::{
    core::Result,
    Foundation::{Numerics::Vector2, TypedEventHandler},
    Win32::{
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG},
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

fn main() -> Result<()> {
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

    let window = Window::new("chartfun", window_width, window_height)?;
    let target = compositor.create_desktop_window_target(window.handle(), false)?;
    target.SetRoot(&root)?;

    let mut chart = ChartSurface::new(&renderer)?;
    let visual = compositor.CreateSpriteVisual()?;
    visual.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    let brush = compositor.CreateSurfaceBrushWithSurface(chart.surface())?;
    brush.SetStretch(CompositionStretch::None)?;
    visual.SetBrush(&brush)?;
    root.Children()?.InsertAtTop(&visual)?;
    chart.redraw(&renderer)?;

    let timer = queue.CreateTimer()?;
    timer.SetInterval(Duration::from_secs(1).into())?;
    timer.SetIsRepeating(true)?;
    let mut last_value = 0.0;
    let timer_token = timer.Tick(&TypedEventHandler::<_, _>::new(move |_, _| -> Result<()> {
        let mut rng = rand::thread_rng();
        let value: f32 = rng.gen_range(-20.0..=20.0);

        let value = (last_value + value).clamp(0.0, 100.0);
        last_value = value;

        chart.add_point(value);
        chart.redraw(&renderer)?;
        Ok(())
    }))?;
    timer.Start()?;

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    timer.RemoveTick(timer_token)?;
    let _ = shutdown_dispatcher_queue_controller_and_wait(&controller, message.wParam.0 as i32)?;
    Ok(())
}
