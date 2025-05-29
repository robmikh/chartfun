use windows::{core::Result, Win32::Foundation::E_FAIL};

use crate::pid::parse_pid;

pub struct Args {
    pub pid: Option<u32>,
    pub adapter: Option<u32>,
}

pub fn parse_args() -> Result<Args> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut args_iter = args.iter();
    let mut pid = None;
    let mut adapter = None;
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--pid" => match args_iter.next() {
                Some(pid_string) => {
                    if let Ok(parsed_pid) = parse_pid(&pid_string) {
                        pid = Some(parsed_pid);
                    } else {
                        return Err(windows::core::Error::new(
                            E_FAIL,
                            "Failed to parse process id!",
                        ));
                    }
                }
                None => {
                    return Err(windows::core::Error::new(
                        E_FAIL,
                        "Expected process id after --pid!",
                    ))
                }
            },
            "--adapter" => match args_iter.next() {
                Some(adapter_string) => {
                    if let Ok(adapter_index) = adapter_string.parse::<u32>() {
                        adapter = Some(adapter_index);
                    } else {
                        return Err(windows::core::Error::new(
                            E_FAIL,
                            "Failed to parse adapter index!",
                        ));
                    }
                }
                None => {
                    return Err(windows::core::Error::new(
                        E_FAIL,
                        "Expected adapter index after --adapter!",
                    ))
                }
            },
            _ => {
                return Err(windows::core::Error::new(
                    E_FAIL,
                    format!("Unknown argument \"{}\"!\nLaunch with no arguments or use arguments like:\n  \"--pid <process id>\"\n  \"--adapter <adapter index>\".", arg),
                ))
            }
        }
    }
    Ok(Args { pid, adapter })
}
