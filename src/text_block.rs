use windows::{
    core::{Result, HSTRING},
    Foundation::Numerics::{Matrix3x2, Vector2},
    Graphics::{
        DirectX::{DirectXAlphaMode, DirectXPixelFormat},
        SizeInt32,
    },
    Win32::{
        Graphics::{
            Direct2D::{
                Common::{D2D1_COLOR_F, D2D_POINT_2F},
                ID2D1DeviceContext, D2D1_DRAW_TEXT_OPTIONS_NONE,
            },
            DirectWrite::IDWriteTextLayout,
        },
        System::WindowsProgramming::MulDiv,
    },
    UI::{
        Color,
        Composition::{CompositionDrawingSurface, CompositionStretch, SpriteVisual},
    },
};

use crate::{
    renderer::Renderer,
    windows_utils::{composition::CompositionDrawingSurfaceInterop, numerics::FromScale},
};

pub struct TextBlock {
    text: String,
    text_layout: IDWriteTextLayout,
    surface: CompositionDrawingSurface,
    root: SpriteVisual,
    dpi: i32,
}

impl TextBlock {
    pub fn new(renderer: &Renderer, text: String, color: Color, dpi: u32) -> Result<Self> {
        let text_layout = unsafe {
            let text = HSTRING::from(&text);
            renderer.dwrite_factory.CreateTextLayout(
                text.as_wide(),
                &renderer.normal_text_format,
                400.0,
                0.0,
            )?
        };

        let metrics = unsafe { text_layout.GetOverhangMetrics()? };
        let (max_width, max_height) = unsafe {
            let width = text_layout.GetMaxWidth();
            let height = text_layout.GetMaxHeight();
            (width, height)
        };
        let text_width = (metrics.right + max_width + -metrics.left).ceil();
        let text_height = (metrics.bottom + max_height + -metrics.top).ceil();
        let text_size = SizeInt32 {
            Width: unsafe { MulDiv(text_width as i32, dpi as i32, 96) },
            Height: unsafe { MulDiv(text_height as i32, dpi as i32, 96) },
        };
        let text_width = text_size.Width as f32;
        let text_height = text_size.Height as f32;

        let root = renderer.compositor.CreateSpriteVisual()?;
        root.SetSize(Vector2::new(text_width, text_height))?;
        let surface_brush = renderer.compositor.CreateSurfaceBrush()?;

        let surface = renderer.comp_graphics.CreateDrawingSurface2(
            text_size,
            DirectXPixelFormat::A8UIntNormalized,
            DirectXAlphaMode::Premultiplied,
        )?;
        surface_brush.SetSurface(&surface)?;
        surface_brush.SetStretch(CompositionStretch::None)?;
        let color_brush = &renderer.black_brush;
        surface.draw::<ID2D1DeviceContext, _>(None, |context, offset| -> Result<()> {
            unsafe {
                context.Clear(Some(&D2D1_COLOR_F {
                    a: 0.0,
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                }));
                context.DrawTextLayout(
                    D2D_POINT_2F {
                        x: offset.x as f32,
                        y: offset.y as f32,
                    },
                    &text_layout,
                    color_brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }
            Ok(())
        })?;

        let mask_brush = renderer.compositor.CreateMaskBrush()?;
        let text_color_brush = renderer.compositor.CreateColorBrushWithColor(color)?;
        mask_brush.SetSource(&text_color_brush)?;
        mask_brush.SetMask(&surface_brush)?;

        root.SetBrush(&mask_brush)?;

        Ok(Self {
            text,
            text_layout,
            surface,
            root,
            dpi: dpi as i32,
        })
    }

    pub fn root(&self) -> &SpriteVisual {
        &self.root
    }

    pub fn set_text(&mut self, renderer: &Renderer, text: String) -> Result<()> {
        self.text = text;
        let text_layout = unsafe {
            let text = HSTRING::from(&self.text);
            renderer.dwrite_factory.CreateTextLayout(
                text.as_wide(),
                &renderer.normal_text_format,
                400.0,
                0.0,
            )?
        };
        self.text_layout = text_layout;
        self.redraw(renderer)?;
        Ok(())
    }

    pub fn redraw(&self, renderer: &Renderer) -> Result<()> {
        let metrics = unsafe { self.text_layout.GetOverhangMetrics()? };
        let (max_width, max_height) = unsafe {
            let width = self.text_layout.GetMaxWidth();
            let height = self.text_layout.GetMaxHeight();
            (width, height)
        };
        let text_width = (metrics.right + max_width + -metrics.left).ceil();
        let text_height = (metrics.bottom + max_height + -metrics.top).ceil();
        let text_size = SizeInt32 {
            Width: unsafe { MulDiv(text_width as i32, self.dpi, 96) },
            Height: unsafe { MulDiv(text_height as i32, self.dpi, 96) },
        };
        let text_width = text_size.Width as f32;
        let text_height = text_size.Height as f32;

        self.root.SetSize(Vector2::new(text_width, text_height))?;

        self.surface.Resize(text_size)?;
        let color_brush = &renderer.black_brush;
        self.surface
            .draw::<ID2D1DeviceContext, _>(None, |context, offset| -> Result<()> {
                unsafe {
                    let transform = Matrix3x2::from_scale(self.dpi as f32 / 96.0)
                        * Matrix3x2::translation(offset.x as f32, offset.y as f32);
                    context.SetTransform(&transform);
                    context.Clear(Some(&D2D1_COLOR_F {
                        a: 0.0,
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    }));
                    context.DrawTextLayout(
                        D2D_POINT_2F { x: 0.0, y: 0.0 },
                        &self.text_layout,
                        color_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }
                Ok(())
            })?;

        Ok(())
    }

    pub fn set_dpi(&mut self, renderer: &Renderer, dpi: u32) -> Result<()> {
        self.dpi = dpi as i32;
        self.redraw(renderer)?;
        Ok(())
    }
}
