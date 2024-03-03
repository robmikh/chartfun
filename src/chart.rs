use std::collections::VecDeque;

use windows::{
    core::Result,
    Foundation::Numerics::Matrix3x2,
    Graphics::{
        DirectX::{DirectXAlphaMode, DirectXPixelFormat},
        SizeInt32,
    },
    Win32::{
        Graphics::Direct2D::{
            Common::{
                D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED, D2D_POINT_2F,
                D2D_RECT_F,
            },
            ID2D1DeviceContext, ID2D1SolidColorBrush,
        },
        System::WindowsProgramming::MulDiv,
    },
    UI::Composition::CompositionDrawingSurface,
};

use crate::{renderer::Renderer, windows_utils::composition::CompositionDrawingSurfaceInterop};

const MAX_POINTS: usize = 60;
const CELL_WIDTH_IN_SECONDS: usize = 10;

pub struct ChartSurface {
    surface: CompositionDrawingSurface,
    points: VecDeque<f32>,
    width: i32,
    height: i32,
    unscaled_width: i32,
    unscaled_height: i32,
    dpi: i32,
    outline_brush: ID2D1SolidColorBrush,
    fill_brush: ID2D1SolidColorBrush,
    grid_brush: ID2D1SolidColorBrush,
    grid_offset: usize,
}

impl ChartSurface {
    pub fn new(renderer: &Renderer, dpi: u32) -> Result<Self> {
        let unscaled_width = 250;
        let unscaled_height = 226;
        let width = unsafe { MulDiv(unscaled_width, dpi as i32, 96) };
        let height = unsafe { MulDiv(unscaled_height, dpi as i32, 96) };
        let surface = renderer.comp_graphics.CreateDrawingSurface2(
            SizeInt32 {
                Width: width,
                Height: height,
            },
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            DirectXAlphaMode::Premultiplied,
        )?;

        let mut color = D2D1_COLOR_F {
            a: 1.0,
            r: 0.0667,
            g: 0.4902,
            b: 0.7333,
        };
        let outline_brush = unsafe { renderer.d2d_context.CreateSolidColorBrush(&color, None)? };
        color.a = 0.1;
        let fill_brush = unsafe { renderer.d2d_context.CreateSolidColorBrush(&color, None)? };
        let grid_brush = unsafe {
            renderer.d2d_context.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    a: 1.0,
                    r: 0.8510,
                    g: 0.9176,
                    b: 0.9569,
                },
                None,
            )?
        };

        Ok(Self {
            surface,
            points: VecDeque::with_capacity(MAX_POINTS),
            width,
            height,
            unscaled_width,
            unscaled_height,
            dpi: dpi as i32,
            outline_brush,
            fill_brush,
            grid_brush,
            grid_offset: 0,
        })
    }

    pub fn surface(&self) -> &CompositionDrawingSurface {
        &self.surface
    }

    pub fn redraw(&self, renderer: &Renderer) -> Result<()> {
        let path_geometry = unsafe { renderer.d2d_factory.CreatePathGeometry()? };
        let pixels_per_second = self.pixels_per_second();
        let pixels_per_percent = self.pixels_per_percent();

        self.surface
            .draw::<ID2D1DeviceContext, _>(None, |context, offset| -> Result<()> {
                unsafe {
                    context.SetTransform(&Matrix3x2::translation(offset.x as f32, offset.y as f32));
                    context.Clear(Some(&D2D1_COLOR_F {
                        a: 0.0,
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    }));

                    // Build geometry
                    {
                        let sink = path_geometry.Open()?;
                        let start_x = (MAX_POINTS - self.points.len()) as f32 * pixels_per_second;
                        sink.BeginFigure(
                            D2D_POINT_2F {
                                x: start_x,
                                y: self.height as f32,
                            },
                            D2D1_FIGURE_BEGIN_FILLED,
                        );

                        let mut last_x = 0.0;
                        for (i, point) in self.points.iter().enumerate() {
                            let x =
                                (start_x + (i as f32 * pixels_per_second)).min(self.width as f32);
                            sink.AddLine(D2D_POINT_2F {
                                x: x,
                                y: self.height as f32 - (point * pixels_per_percent),
                            });
                            last_x = x;
                        }

                        sink.AddLine(D2D_POINT_2F {
                            x: last_x,
                            y: self.height as f32,
                        });

                        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        sink.Close()?;
                    }

                    // Draw graph lines
                    // Horizontal lines
                    let cell_height = self.height as f32 / 10.0;
                    for i in 0..9 {
                        let y = (i + 1) as f32 * cell_height;
                        context.DrawLine(
                            D2D_POINT_2F { x: 0.0, y: y },
                            D2D_POINT_2F {
                                x: self.width as f32,
                                y: y,
                            },
                            &self.grid_brush,
                            1.0,
                            None,
                        );
                    }
                    // Vertical lines
                    let cell_width = CELL_WIDTH_IN_SECONDS as f32 * pixels_per_second;
                    for i in 0..10 {
                        let offset = self.grid_offset as f32 * pixels_per_second;
                        let x = (i + 1) as f32 * cell_width;
                        context.DrawLine(
                            D2D_POINT_2F {
                                x: x - offset,
                                y: 0.0,
                            },
                            D2D_POINT_2F {
                                x: x - offset,
                                y: self.height as f32,
                            },
                            &self.grid_brush,
                            1.0,
                            None,
                        );
                    }

                    context.DrawGeometry(&path_geometry, &self.outline_brush, 1.0, None);
                    context.FillGeometry(&path_geometry, &self.fill_brush, None);

                    context.DrawRectangle(
                        &D2D_RECT_F {
                            left: 0.0,
                            top: 0.0,
                            right: self.width as f32,
                            bottom: self.height as f32,
                        },
                        &self.outline_brush,
                        2.0,
                        None,
                    );
                }
                Ok(())
            })?;
        Ok(())
    }

    pub fn add_point(&mut self, point: f32) {
        if self.points.len() == MAX_POINTS {
            self.points.pop_front();
        }
        self.points.push_back(point);
        self.grid_offset = (self.grid_offset + 1) % CELL_WIDTH_IN_SECONDS;
    }

    pub fn size(&self) -> SizeInt32 {
        SizeInt32 {
            Width: self.width,
            Height: self.height,
        }
    }

    pub fn set_dpi(&mut self, renderer: &Renderer, dpi: u32) -> Result<()> {
        self.dpi = dpi as i32;
        self.width = unsafe { MulDiv(self.unscaled_width, self.dpi, 96) };
        self.height = unsafe { MulDiv(self.unscaled_height, self.dpi, 96) };

        self.surface.Resize(self.size())?;
        self.redraw(renderer)?;

        Ok(())
    }

    fn pixels_per_second(&self) -> f32 {
        self.width as f32 / (MAX_POINTS - 1) as f32
    }

    fn pixels_per_percent(&self) -> f32 {
        self.height as f32 / 100.0
    }
}
