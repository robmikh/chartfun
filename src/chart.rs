use std::collections::VecDeque;

use windows::{core::Result, Foundation::Numerics::Matrix3x2, Graphics::{DirectX::{DirectXAlphaMode, DirectXPixelFormat}, SizeInt32}, Win32::Graphics::Direct2D::{Common::{D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_END_CLOSED, D2D_POINT_2F, D2D_RECT_F}, ID2D1DeviceContext}, UI::Composition::CompositionDrawingSurface};

use crate::{renderer::Renderer, windows_utils::composition::CompositionDrawingSurfaceInterop};

const MAX_POINTS: usize = 60;

pub struct ChartSurface {
    surface: CompositionDrawingSurface,
    points: VecDeque<f32>,
    width: i32,
    height: i32,
}

impl ChartSurface {
    pub fn new(renderer: &Renderer) -> Result<Self> {
        let width = 250;
        let height = 225;
        let surface = renderer.comp_graphics.CreateDrawingSurface2(
            SizeInt32 { Width: width, Height: height }, 
            DirectXPixelFormat::B8G8R8A8UIntNormalized, 
            DirectXAlphaMode::Premultiplied
        )?;

        Ok(Self { 
            surface, 
            points: VecDeque::with_capacity(MAX_POINTS),
            width,
            height,
        })
    }

    pub fn surface(&self) -> &CompositionDrawingSurface {
        &self.surface
    }

    pub fn redraw(&self, renderer: &Renderer) -> Result<()> {
        let path_geometry = unsafe { renderer.d2d_factory.CreatePathGeometry()? };
        let pixels_per_second = self.pixels_per_second();
        let pixels_per_percent = self.pixels_per_percent();

        self.surface.draw::<ID2D1DeviceContext, _>(None, |context, offset| -> Result<()> {
            unsafe {
                context.SetTransform(&Matrix3x2::translation(offset.x as f32, offset.y as f32));
                context.Clear(Some(&D2D1_COLOR_F{a: 0.0, r: 0.0, g: 0.0, b: 0.0 }));

                // Build geometry
                {
                    let sink = path_geometry.Open()?;
                    let start_x = (MAX_POINTS - self.points.len()) as f32 * pixels_per_second;
                    sink.BeginFigure(D2D_POINT_2F { x: start_x, y: self.height as f32 }, D2D1_FIGURE_BEGIN_FILLED);
    
                    let mut current_x = start_x;
                    for point in &self.points {
                        sink.AddLine(D2D_POINT_2F { x: current_x, y: self.height as f32 - (point * pixels_per_percent)});
                        current_x += pixels_per_second;
                    }
                    //println!("current_x: {}", current_x);
                    //println!("pixels_per_second: {}", pixels_per_second);
                    //println!("");
    
                    sink.AddLine(D2D_POINT_2F { x: self.width as f32, y: self.height as f32 });
    
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                    sink.Close()?;
                }

                context.DrawRectangle(&D2D_RECT_F { left: 0.0, top: 0.0, right: self.width as f32, bottom: self.height as f32}, &renderer.black_brush, 2.0, None);

                // TODO: Draw graph lines

                context.FillGeometry(&path_geometry, &renderer.black_brush, None);
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
    }

    fn pixels_per_second(&self) -> f32 {
        self.width as f32 / MAX_POINTS as f32
    }

    fn pixels_per_percent(&self) -> f32 {
        self.height as f32 / 100.0
    }
}