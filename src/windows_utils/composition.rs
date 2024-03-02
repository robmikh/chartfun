use windows::{
    core::{CanInto, IUnknown, Interface, Result},
    Win32::{
        Foundation::{ERROR_SPACES_UPDATE_COLUMN_STATE, HWND, POINT, RECT},
        System::WinRT::Composition::{
            ICompositionDrawingSurfaceInterop, ICompositorDesktopInterop, ICompositorInterop,
        },
    },
    UI::Composition::{
        CompositionDrawingSurface, CompositionGraphicsDevice, Compositor,
        Desktop::DesktopWindowTarget,
    },
};

pub trait CompositionInterop {
    fn create_desktop_window_target(
        &self,
        window: HWND,
        is_topmost: bool,
    ) -> Result<DesktopWindowTarget>;
    fn create_graphics_device<T: Interface + CanInto<IUnknown>>(
        &self,
        rendering_device: &T,
    ) -> Result<CompositionGraphicsDevice>;
}

impl CompositionInterop for Compositor {
    fn create_desktop_window_target(
        &self,
        window: HWND,
        is_topmost: bool,
    ) -> Result<DesktopWindowTarget> {
        let compositor_desktop: ICompositorDesktopInterop = self.cast()?;
        unsafe { compositor_desktop.CreateDesktopWindowTarget(window, is_topmost) }
    }

    fn create_graphics_device<T: Interface + CanInto<IUnknown>>(
        &self,
        rendering_device: &T,
    ) -> Result<CompositionGraphicsDevice> {
        let interop: ICompositorInterop = self.cast()?;
        unsafe { interop.CreateGraphicsDevice(rendering_device) }
    }
}

pub trait CompositionDrawingSurfaceInterop {
    fn draw<T: Interface, F: FnOnce(T, POINT) -> Result<()>>(
        &self,
        update_rect: Option<RECT>,
        draw_fn: F,
    ) -> Result<()>;
}

impl CompositionDrawingSurfaceInterop for CompositionDrawingSurface {
    fn draw<T: Interface, F: FnOnce(T, POINT) -> Result<()>>(
        &self,
        update_rect: Option<RECT>,
        draw_fn: F,
    ) -> Result<()> {
        let interop: ICompositionDrawingSurfaceInterop = self.cast()?;
        let update_rect = update_rect.map(|x| &x as *const _);
        let mut offset = POINT::default();
        let context: T = unsafe { interop.BeginDraw(update_rect, &mut offset)? };

        let result = draw_fn(context, offset);

        unsafe {
            interop.EndDraw()?;
        }
        result
    }
}
