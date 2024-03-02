use windows::{core::{w, Result}, Graphics::{DirectX::{DirectXAlphaMode, DirectXPixelFormat}, SizeInt32}, Win32::{Graphics::{Direct2D::{Common::{D2D1_COLOR_F, D2D_RECT_F}, ID2D1Device, ID2D1Device1, ID2D1DeviceContext, ID2D1Factory, ID2D1Factory1, ID2D1SolidColorBrush, D2D1_DEVICE_CONTEXT_OPTIONS, D2D1_DEVICE_CONTEXT_OPTIONS_NONE, D2D1_ROUNDED_RECT}, Direct3D11::{ID3D11Device, ID3D11DeviceContext}, DirectWrite::{IDWriteFactory, IDWriteFontCollection, IDWriteRenderingParams, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_FACE_TYPE_OPENTYPE_COLLECTION, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL}, Gdi::{MonitorFromWindow, MONITOR_DEFAULTTOPRIMARY}}, UI::WindowsAndMessaging::GetDesktopWindow}, UI::Composition::{CompositionDrawingSurface, CompositionGraphicsDevice, CompositionNineGridBrush, CompositionStretch, CompositionSurfaceBrush, Compositor}};

use crate::windows_utils::{composition::{CompositionDrawingSurfaceInterop, CompositionInterop}, d2d::{create_d2d_device, create_d2d_factory}, d3d::create_d3d_device, dwrite::create_dwrite_factory};

pub struct Renderer {
    pub d3d_device: ID3D11Device,
    pub d3d_context: ID3D11DeviceContext,
    pub d2d_factory: ID2D1Factory1,
    pub d2d_device: ID2D1Device,
    pub d2d_context: ID2D1DeviceContext,
    pub compositor: Compositor,
    pub comp_graphics: CompositionGraphicsDevice,
    pub dwrite_factory: IDWriteFactory,
    pub font_collection: IDWriteFontCollection,

    // TODO: D2D brush storage
    pub black_brush: ID2D1SolidColorBrush,
    
    // TODO: Text format storage
    pub normal_text_format: IDWriteTextFormat,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let compositor = Compositor::new()?;
        let d3d_device = create_d3d_device()?;
        let d3d_context = unsafe { d3d_device.GetImmediateContext()? };
        let d2d_factory = create_d2d_factory()?;
        let d2d_device = create_d2d_device(&d2d_factory, &d3d_device)?;
        let d2d_context = unsafe {
            d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?
        };
        let comp_graphics = compositor.create_graphics_device(&d2d_device)?;

        let dwrite_factory = create_dwrite_factory(DWRITE_FACTORY_TYPE_SHARED)?;
        let font_collection = unsafe {
            let mut collection = None;
            dwrite_factory.GetSystemFontCollection(&mut collection, false)?;
            collection.unwrap()
        };

        let black_brush = 
            unsafe {
                d2d_context.CreateSolidColorBrush(&D2D1_COLOR_F{ a: 1.0, r: 0.0, g: 0.0, b: 0.0 }, None)?
            };

        let normal_text_format = unsafe {
            dwrite_factory.CreateTextFormat(
                w!("Segoe UI"), 
                &font_collection, 
                DWRITE_FONT_WEIGHT_NORMAL, 
                DWRITE_FONT_STYLE_NORMAL, 
                DWRITE_FONT_STRETCH_NORMAL, 
                14.0, 
            w!("en-us"))?
        };

        Ok(Self {
            d3d_device,
            d3d_context,
            d2d_factory,
            d2d_device,
            d2d_context,
            compositor,
            comp_graphics,
            dwrite_factory,
            font_collection,
            black_brush,
            normal_text_format,
        })
    }
}