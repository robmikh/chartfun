use crate::{perf::PerfTracker, pid::get_name_from_pid, sources::DataSource};

use windows::{core::Result, Win32::Foundation::LUID};

pub struct GpuUtilization {
    pid: u32,
    perf_tracker: PerfTracker,
}

impl GpuUtilization {
    pub fn new(process_id: u32, luid: Option<LUID>) -> Result<Self> {
        let perf_tracker = PerfTracker::new(process_id, luid)?;
        Ok(Self {
            pid: process_id,
            perf_tracker,
        })
    }
}

impl DataSource for GpuUtilization {
    fn start(&self) -> Result<()> {
        self.perf_tracker.start()
    }

    fn close(&mut self) -> Result<()> {
        self.perf_tracker.close()
    }

    fn get_current_value(&self) -> Result<f64> { 
        self.perf_tracker.get_current_value()
    }

    fn gen_current_max(&self) -> Result<f64> {
        Ok(100.0)
    }
    
    fn label(&self) -> Result<String> {
        get_name_from_pid(self.pid)
    }
    
    fn unit_label(&self) -> &str {
        "%"
    }
}