use processdumper::{find_process_id_with_name_in_session, get_session_for_current_process};
use windows::Win32::Foundation::E_FAIL;

pub fn parse_pid(value: &str) -> Result<u32, std::num::ParseIntError> {
    if value.starts_with("0x") {
        u32::from_str_radix(value.trim_start_matches("0x"), 16)
    } else {
        value.parse()
    }
}

pub fn get_current_dwm_pid() -> windows::core::Result<u32> {
    // During RDP sessions, you'll have multiple sessions and muiltple
    // DWMs. We want the one the user is currently using, so find the
    // session our program is running in.
    let current_session = get_session_for_current_process()?;
    let process_id = if let Some(process_id) =
        find_process_id_with_name_in_session("dwm.exe", current_session)?
    {
        process_id
    } else {
        return Err(windows::core::Error::new(
            E_FAIL,
            "Could not find a dwm process for this session!",
        ));
    };
    Ok(process_id)
}
