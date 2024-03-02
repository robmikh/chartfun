pub fn parse_pid(value: &str) -> Result<u32, std::num::ParseIntError> {
    if value.starts_with("0x") {
        u32::from_str_radix(value.trim_start_matches("0x"), 16)
    } else {
        value.parse()
    }
}
