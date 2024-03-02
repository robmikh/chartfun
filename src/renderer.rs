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

    // TODO: Common asset storage
    pub small_rounded_corner_surface: CompositionDrawingSurface,
    pub small_rounded_corner_mask_surface: CompositionDrawingSurface,
    pub small_rounded_corner_brush: CompositionNineGridBrush,
    pub small_rounded_corner_mask_brush: CompositionSurfaceBrush,
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

        let small_rounded_corner_surface = comp_graphics.CreateDrawingSurface2(SizeInt32 { Width: 16, Height: 16}, DirectXPixelFormat::B8G8R8A8UIntNormalized, DirectXAlphaMode::Premultiplied)?;
        let small_rounded_corner_mask_surface = comp_graphics.CreateDrawingSurface2(SizeInt32 { Width: 16, Height: 16}, DirectXPixelFormat::B8G8R8A8UIntNormalized, DirectXAlphaMode::Premultiplied)?;

        let small_corner_brush = compositor.CreateSurfaceBrushWithSurface(&small_rounded_corner_surface)?;
        small_corner_brush.SetStretch(CompositionStretch::Fill)?;
        let small_rounded_corner_brush = compositor.CreateNineGridBrush()?;
        small_rounded_corner_brush.SetInsets(4.0)?;
        small_rounded_corner_brush.SetIsCenterHollow(true)?;
        small_rounded_corner_brush.SetSource(&small_corner_brush)?;
        let small_rounded_rect = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: 1.5,
                top: 1.5,
                right: 13.5,
                bottom: 13.5,
            },
            radiusX: 4.0,
            radiusY: 4.0,
        };
        small_rounded_corner_surface.draw::<ID2D1DeviceContext, _>(None, |context, offset| -> Result<()> {
            unsafe {
                let mut rect = small_rounded_rect;
                rect.rect.left += offset.x as f32;
                rect.rect.top += offset.y as f32;
                rect.rect.right += offset.x as f32;
                rect.rect.bottom += offset.y as f32;

                context.Clear(Some(&D2D1_COLOR_F{a: 0.0, r: 0.0, g: 0.0, b: 0.0 }));
                context.DrawRoundedRectangle(&rect, &black_brush, 0.5, None);
            }
            Ok(())
        })?;

        let small_rounded_corner_mask_brush = compositor.CreateSurfaceBrushWithSurface(&small_rounded_corner_mask_surface)?;
        small_rounded_corner_mask_brush.SetStretch(CompositionStretch::Fill)?;
        small_rounded_corner_mask_surface.draw::<ID2D1DeviceContext, _>(None, |context, offset| -> Result<()> {
            unsafe {
                let mut rect = small_rounded_rect;
                rect.rect.left += offset.x as f32;
                rect.rect.top += offset.y as f32;
                rect.rect.right += offset.x as f32;
                rect.rect.bottom += offset.y as f32;

                context.Clear(Some(&D2D1_COLOR_F{a: 0.0, r: 0.0, g: 0.0, b: 0.0 }));
                context.FillRoundedRectangle(&rect, &black_brush);
            }
            Ok(())
        })?;

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
            small_rounded_corner_surface,
            small_rounded_corner_mask_surface,
            small_rounded_corner_brush,
            small_rounded_corner_mask_brush,
        })
    }
}