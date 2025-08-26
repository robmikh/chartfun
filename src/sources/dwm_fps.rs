use std::time::Duration;

use crate::{perf::PerfTracker, pid::get_name_from_pid, sources::DataSource};

use windows::{core::Result, Foundation::TimeSpan, Win32::{Foundation::{E_FAIL, LUID}, Graphics::DirectComposition::{DCompositionGetFrameId, DCompositionGetStatistics, DCompositionGetTargetStatistics, COMPOSITION_FRAME_ID_CONFIRMED, COMPOSITION_FRAME_STATS, COMPOSITION_TARGET_ID, COMPOSITION_TARGET_STATS}}};

pub struct DwmFps {
    display_adapter_luid: LUID,
}

impl DwmFps {
    pub fn new(luid: LUID) -> Result<Self> {
        Ok(Self {
            display_adapter_luid: luid,
        })
    }
}

impl DataSource for DwmFps {
    fn start(&self) -> Result<()> {
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_current_value(&self) -> Result<f64> { 
        // First we need to get the frame id
        let frame_id = unsafe {
            DCompositionGetFrameId(COMPOSITION_FRAME_ID_CONFIRMED)?
        };

        // Next the the info for this frame so that we can find our target
        let (frame_stats, target_ids) = unsafe {
            let mut frame_stats = COMPOSITION_FRAME_STATS::default();
            let mut actual_target_count = 0;
            DCompositionGetStatistics(
                frame_id, 
                &mut frame_stats,
                0,
                None,
                 Some(&mut actual_target_count))?;
            let mut target_ids = vec![COMPOSITION_TARGET_ID::default(); actual_target_count as usize];
            DCompositionGetStatistics(
                frame_id, 
                &mut frame_stats,
                target_ids.len() as u32,
                Some(target_ids.as_mut_ptr()),
                 Some(&mut actual_target_count))?;
            (frame_stats, target_ids)
        };
        //println!("num targets: {}", target_ids.len());

        // Frame period is in 100 nano second units, same as TimeSpan
        let frame_period: Duration = TimeSpan{ Duration: frame_stats.framePeriod as i64 }.into();
        let frame_period_in_seconds = frame_period.as_secs_f64();
        let frames_per_second = 1.0 / frame_period_in_seconds;

        /*
        // Find the target we're after
        let target_id = match target_ids.iter().find(|x| x.displayAdapterLuid == self.display_adapter_luid) {
            Some(target_id) => target_id,
            None => return Err(windows::core::Error::new(
                                E_FAIL,
                                "Couldn't find adapter!",
                            ))
        };
        let target_stats = unsafe {
            let mut target_stats = COMPOSITION_TARGET_STATS::default();
            DCompositionGetTargetStatistics(frame_id, target_id, &mut target_stats)?;
            target_stats
        };

        // Frame period is in 100 nano second units, same as TimeSpan
        let vblank_duration: Duration = TimeSpan{ Duration: target_stats.vblankDuration as i64 }.into();
        let vblank_duration_in_seconds = vblank_duration.as_secs_f64();
        let vblanks_per_second = 1.0 / vblank_duration_in_seconds;
        */

        Ok(frames_per_second)
    }

    fn gen_current_max(&self) -> Result<f64> {
        Ok(200.0 / 100.0)
    }
    
    fn label(&self) -> Result<String> {
        Ok("dwm.exe".to_owned())
    }
    
    fn unit_label(&self) -> &str {
        " fps"
    }
}