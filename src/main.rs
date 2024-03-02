mod renderer;
mod windows_utils;
mod window;
mod chart;

use chart::ChartSurface;
use renderer::Renderer;
use window::Window;
use windows::{core::Result, Foundation::Numerics::{Vector2, Vector3}, Win32::{System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED}, UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG}}, UI::{Color, Composition::{CompositionStretch, Compositor}}};
use windows_utils::{composition::CompositionInterop, dispatcher_queue::{create_dispatcher_queue_controller_for_current_thread, shutdown_dispatcher_queue_controller_and_wait}};

fn main() -> Result<()> {
    unsafe { RoInitialize(RO_INIT_SINGLETHREADED)? };
    let controller = create_dispatcher_queue_controller_for_current_thread()?;

    let window_width = 800;
    let window_height = 600;

    let renderer = Renderer::new()?;

    let compositor = renderer.compositor.clone();
    let root = compositor.CreateSpriteVisual()?;
    root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;
    root.SetBrush(&compositor.CreateColorBrushWithColor(Color { A: 255, R: 255, G: 255, B: 255 })?)?;

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

    chart.add_point(0.05 * 100.0);
    chart.add_point(0.15 * 100.0);
    chart.add_point(0.50 * 100.0);
    chart.add_point(0.35 * 100.0);
    chart.redraw(&renderer)?;

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    let _ = shutdown_dispatcher_queue_controller_and_wait(&controller, message.wParam.0 as i32)?;
    Ok(())
}
