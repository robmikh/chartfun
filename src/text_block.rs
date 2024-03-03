use windows::{
    core::{Result, HSTRING},
    Foundation::Numerics::Vector2,
    Graphics::{
        DirectX::{DirectXAlphaMode, DirectXPixelFormat},
        SizeInt32,
    },
    Win32::Graphics::{
        Direct2D::{
            Common::{D2D1_COLOR_F, D2D_POINT_2F},
            ID2D1DeviceContext, D2D1_DRAW_TEXT_OPTIONS_NONE,
        },
        DirectWrite::IDWriteTextLayout,
    },
    UI::{
        Color,
        Composition::{CompositionDrawingSurface, CompositionStretch, SpriteVisual},
    },
};

use crate::{renderer::Renderer, windows_utils::composition::CompositionDrawingSurfaceInterop};

pub struct TextBlock {
    text: String,
    text_layout: IDWriteTextLayout,
    surface: CompositionDrawingSurface,
    root: SpriteVisual,
}

impl TextBlock {
    pub fn new(renderer: &Renderer, text: String, color: Color) -> Result<Self> {
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
            Width: text_width as i32,
            Height: text_height as i32,
        };

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

        let metrics = unsafe { self.text_layout.GetOverhangMetrics()? };
        let (max_width, max_height) = unsafe {
            let width = self.text_layout.GetMaxWidth();
            let height = self.text_layout.GetMaxHeight();
            (width, height)
        };
        let text_width = (metrics.right + max_width + -metrics.left).ceil();
        let text_height = (metrics.bottom + max_height + -metrics.top).ceil();
        let text_size = SizeInt32 {
            Width: text_width as i32,
            Height: text_height as i32,
        };

        self.root.SetSize(Vector2::new(text_width, text_height))?;

        self.surface.Resize(text_size)?;
        let color_brush = &renderer.black_brush;
        self.surface
            .draw::<ID2D1DeviceContext, _>(None, |context, offset| -> Result<()> {
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
                        &self.text_layout,
                        color_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }
                Ok(())
            })?;

        Ok(())
    }
}
