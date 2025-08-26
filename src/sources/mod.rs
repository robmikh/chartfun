use windows::core::Result;

pub trait DataSource {
    fn start(&self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn get_current_value(&self) -> Result<f64>;
    fn gen_current_max(&self) -> Result<f64>;
    fn label(&self) -> Result<String>;
    fn unit_label(&self) -> &str;
}


pub mod gpu_util;
pub mod dwm_fps;