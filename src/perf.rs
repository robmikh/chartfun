use windows::{
    core::Result,
    Win32::System::Performance::{
        PdhCollectQueryData, PdhGetFormattedCounterValue, PDH_CSTATUS_VALID_DATA,
        PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
    },
};

use crate::pdh::{add_perf_counters, PerfQueryHandle, PDH_FUNCTION};

pub struct PerfTracker {
    query_handle: PerfQueryHandle,
    counter_handles: Vec<isize>,
}

impl PerfTracker {
    pub fn new(process_id: u32) -> Result<Self> {
        let counter_path = format!(
            r#"\GPU Engine(pid_{}*engtype_3D)\Utilization Percentage"#,
            process_id
        );

        let query_handle = PerfQueryHandle::open_query()?;
        let counter_handles = add_perf_counters(&query_handle, &counter_path)?;

        Ok(Self {
            query_handle,
            counter_handles,
        })
    }

    pub fn start(&self) -> Result<()> {
        self.collect_query_data()
    }

    pub fn get_current_value(&self) -> Result<f64> {
        self.collect_query_data()?;

        let mut utilization_value = 0.0;
        for counter_handle in &self.counter_handles {
            let counter_value = unsafe {
                let mut counter_type = 0;
                let mut counter_value = PDH_FMT_COUNTERVALUE::default();
                PDH_FUNCTION(PdhGetFormattedCounterValue(
                    *counter_handle,
                    PDH_FMT_DOUBLE,
                    Some(&mut counter_type),
                    &mut counter_value,
                ))
                .ok()?;
                counter_value
            };
            assert_eq!(counter_value.CStatus, PDH_CSTATUS_VALID_DATA);
            let value = unsafe { counter_value.Anonymous.doubleValue };
            utilization_value += value;
        }
        Ok(utilization_value)
    }

    pub fn close(mut self) -> Result<()> {
        self.query_handle.close_query()
    }

    fn collect_query_data(&self) -> Result<()> {
        unsafe { PDH_FUNCTION(PdhCollectQueryData(self.query_handle.0)).ok() }
    }
}
