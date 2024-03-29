use std::sync::Once;

use windows::{
    core::{w, Result, HSTRING, PCWSTR},
    Foundation::Numerics::Vector2,
    Graphics::SizeInt32,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, WindowsProgramming::MulDiv},
        UI::{
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetWindowLongPtrW,
                LoadCursorW, PostQuitMessage, RegisterClassW, SetWindowLongPtrW, SetWindowPos,
                ShowWindow, CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, SWP_NOACTIVATE,
                SWP_NOMOVE, SWP_NOZORDER, SW_SHOW, WM_DESTROY, WM_DPICHANGED, WM_LBUTTONDOWN,
                WM_MOUSEMOVE, WM_NCCREATE, WM_RBUTTONDOWN, WM_SIZE, WM_SIZING, WNDCLASSW,
                WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

use crate::{app::App, windows_utils::handle::CheckHandle};

static REGISTER_WINDOW_CLASS: Once = Once::new();
const WINDOW_CLASS_NAME: PCWSTR = w!("chartfun.Window");

pub struct Window {
    handle: HWND,
    app: Option<Box<App>>,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Box<Self>> {
        let instance = unsafe { GetModuleHandleW(None)? };
        REGISTER_WINDOW_CLASS.call_once(|| {
            let class = WNDCLASSW {
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW).ok().unwrap() },
                hInstance: instance.into(),
                lpszClassName: WINDOW_CLASS_NAME,
                lpfnWndProc: Some(Self::wnd_proc),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });

        let window_ex_style = WS_EX_NOREDIRECTIONBITMAP;
        let window_style = WS_OVERLAPPEDWINDOW;

        let mut result = Box::new(Self {
            handle: HWND(0),
            app: None,
        });
        let window = unsafe {
            CreateWindowExW(
                window_ex_style,
                WINDOW_CLASS_NAME,
                &HSTRING::from(title),
                window_style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                None,
                None,
                instance,
                Some(result.as_mut() as *mut _ as _),
            )
            .ok()?
        };

        let dpi = unsafe { GetDpiForWindow(window) };

        let (adjusted_width, adjusted_height) = {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            unsafe {
                AdjustWindowRectExForDpi(&mut rect, window_style, false, window_ex_style, dpi)?;
            }
            (rect.right - rect.left, rect.bottom - rect.top)
        };
        // For some reason, we will get scaled *down* by the dpi instead of using the values we pass into SetWindowPos...?
        let (adjusted_width, adjusted_height) = unsafe {
            (
                MulDiv(adjusted_width, dpi as i32, 96),
                MulDiv(adjusted_height, dpi as i32, 96),
            )
        };
        unsafe {
            SetWindowPos(
                window,
                None,
                0,
                0,
                adjusted_width,
                adjusted_height,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
            )?;
        }

        Ok(result)
    }

    pub fn size(&self) -> Result<SizeInt32> {
        get_window_size(self.handle)
    }

    pub fn handle(&self) -> HWND {
        self.handle
    }

    pub fn show(&self) {
        unsafe { ShowWindow(self.handle, SW_SHOW) };
    }

    pub fn dpi(&self) -> u32 {
        unsafe { GetDpiForWindow(self.handle) }
    }

    pub fn set_app(&mut self, app: Box<App>) {
        self.app = Some(app);
    }

    pub fn shutdown_app(&mut self) -> Result<()> {
        let app = self.app.take();
        if let Some(app) = app {
            app.shutdown()?;
        }
        Ok(())
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                return LRESULT(0);
            }
            WM_MOUSEMOVE => {
                let (x, y) = get_mouse_position(lparam);
                let _point = Vector2 {
                    X: x as f32,
                    Y: y as f32,
                };
                //self.game.on_pointer_moved(&point).unwrap();
            }
            WM_SIZE | WM_SIZING => {
                let new_size = self.size().unwrap();
                let _new_size = Vector2 {
                    X: new_size.Width as f32,
                    Y: new_size.Height as f32,
                };
                //self.game.on_parent_size_changed(&new_size).unwrap();
            }
            WM_LBUTTONDOWN => {
                //self.game.on_pointer_pressed(false, false).unwrap();
            }
            WM_RBUTTONDOWN => {
                //self.game.on_pointer_pressed(true, false).unwrap();
            }
            WM_DPICHANGED => {
                let rect: *const RECT = unsafe { std::mem::transmute(lparam) };
                let rect = unsafe { rect.as_ref().unwrap() };
                let _ = unsafe {
                    SetWindowPos(
                        self.handle,
                        None,
                        rect.left,
                        rect.top,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    )
                };
                let dpi = self.dpi();
                if let Some(app) = self.app.as_mut() {
                    app.on_dpi_changed(dpi).unwrap();
                }
                return LRESULT(0);
            }
            _ => {}
        }
        unsafe { DefWindowProcW(self.handle, message, wparam, lparam) }
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCCREATE {
            let cs = lparam.0 as *const CREATESTRUCTW;
            let this = (*cs).lpCreateParams as *mut Self;
            (*this).handle = window;

            SetWindowLongPtrW(window, GWLP_USERDATA, this as _);
        } else {
            let this = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Self;

            if let Some(this) = this.as_mut() {
                return this.message_handler(message, wparam, lparam);
            }
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}

fn get_window_size(window_handle: HWND) -> Result<SizeInt32> {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(window_handle, &mut rect)?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        Ok(SizeInt32 {
            Width: width,
            Height: height,
        })
    }
}

fn get_mouse_position(lparam: LPARAM) -> (isize, isize) {
    let x = lparam.0 & 0xffff;
    let y = (lparam.0 >> 16) & 0xffff;
    (x, y)
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = unsafe { DestroyWindow(self.handle) };
    }
}
