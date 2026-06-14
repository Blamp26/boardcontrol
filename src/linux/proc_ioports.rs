use std::fs;

use crate::error::{Error, Result};

pub fn parse_superio_004e_004f_available(contents: &str) -> bool {
    !contents.to_ascii_lowercase().contains("004e-004f")
}

pub fn superio_ports_available() -> Result<bool> {
    let contents = fs::read_to_string("/proc/ioports")
        .map_err(|err| Error::ProcIoportsReadFailed(err.to_string()))?;
    Ok(parse_superio_004e_004f_available(&contents))
}

#[cfg(test)]
mod tests {
    use super::parse_superio_004e_004f_available;

    #[test]
    fn unavailable_when_range_is_present() {
        let contents = "0000-0000 : foo\n004e-004f : Super I/O\n";
        assert!(!parse_superio_004e_004f_available(contents));
    }

    #[test]
    fn available_when_range_is_missing() {
        let contents = "0000-0000 : foo\n0060-006f : keyboard\n";
        assert!(parse_superio_004e_004f_available(contents));
    }

    #[test]
    fn available_when_other_ranges_exist() {
        let contents = "0060-006f : keyboard\n03f8-03ff : serial\n";
        assert!(parse_superio_004e_004f_available(contents));
    }
}
