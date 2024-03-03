use windows::{
    Foundation::Numerics::{Matrix3x2, Vector2},
    Graphics::SizeInt32,
};

pub trait ToVector2 {
    fn to_vector2(self) -> Vector2;
}

impl ToVector2 for SizeInt32 {
    fn to_vector2(self) -> Vector2 {
        Vector2::new(self.Width as f32, self.Height as f32)
    }
}

pub trait FromScale {
    fn from_scale(scale: f32) -> Self;
}

impl FromScale for Matrix3x2 {
    fn from_scale(scale: f32) -> Self {
        Self {
            M11: scale,
            M12: 0.0,
            M21: 0.0,
            M22: scale,
            M31: 0.0,
            M32: 0.0,
        }
    }
}
