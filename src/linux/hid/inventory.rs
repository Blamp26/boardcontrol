use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub const TARGET_VENDOR_ID: u16 = 0x0DB0;
pub const TARGET_PRODUCT_ID: u16 = 0x0076;
pub const TARGET_INTERFACE_NUMBER: u8 = 0;
pub const TARGET_COLLECTION_NUMBER: u8 = 0;
pub const TARGET_BOARD_ID: u16 = 0x7E75;
pub const DEFAULT_SYSFS_HID_ROOT: &str = "/sys/bus/hid/devices";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidInventoryCandidate {
    pub syspath: PathBuf,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: Option<u8>,
    pub collection_number: Option<u8>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
    pub serial_prefix: SerialPrefixStatus,
}

impl HidInventoryCandidate {
    pub fn is_plausible_ms7e75(&self) -> bool {
        matches!(
            self.serial_prefix,
            SerialPrefixStatus::ExpectedBoardId(TARGET_BOARD_ID) | SerialPrefixStatus::Missing
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerialPrefixStatus {
    Missing,
    Invalid(String),
    UnexpectedBoardId(u16),
    ExpectedBoardId(u16),
}

impl SerialPrefixStatus {
    pub fn summary(&self) -> String {
        match self {
            Self::Missing => "unknown (serial missing or unreadable)".to_string(),
            Self::Invalid(serial) => format!("invalid (first 4 chars are not hex: {serial})"),
            Self::UnexpectedBoardId(board_id) => {
                format!("unexpected board id 0x{board_id:04X}")
            }
            Self::ExpectedBoardId(board_id) => format!("expected board id 0x{board_id:04X}"),
        }
    }
}

pub fn inventory_candidates() -> Result<Vec<HidInventoryCandidate>> {
    inventory_candidates_from_root(Path::new(DEFAULT_SYSFS_HID_ROOT))
}

pub fn inventory_candidates_from_root(root: &Path) -> Result<Vec<HidInventoryCandidate>> {
    let mut candidates = Vec::new();
    let entries = fs::read_dir(root).map_err(|err| {
        Error::HidInventoryReadFailed(format!("unable to read {}: {err}", root.display()))
    })?;

    for entry in entries {
        let entry = entry.map_err(|err| {
            Error::HidInventoryReadFailed(format!("unable to enumerate {}: {err}", root.display()))
        })?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let metadata = parse_candidate_metadata(&path)?;
        if matches_target_candidate(&metadata) {
            candidates.push(metadata);
        }
    }

    candidates.sort_by(|left, right| left.syspath.cmp(&right.syspath));
    Ok(candidates)
}

pub fn format_inventory_report(candidates: &[HidInventoryCandidate]) -> String {
    let mut lines = vec![
        "MS-7E75 HID inventory".to_string(),
        "  status = READ ONLY".to_string(),
        "  mode = read-only metadata scan only".to_string(),
        "  support = unsupported/not enabled".to_string(),
        "  devices_opened = no".to_string(),
        "  writes_enabled = no".to_string(),
        "  writes_performed = no".to_string(),
        "  message = metadata only; no HID device opened and no report write attempted".to_string(),
        "  next_safe_command = msi-ml linux hid gate".to_string(),
    ];

    if candidates.is_empty() {
        lines.push("  candidates = 0".to_string());
        lines.push("  result = no matching MSI common HID candidates found".to_string());
        return lines.join("\n");
    }

    lines.push(format!("  candidates = {}", candidates.len()));
    for (index, candidate) in candidates.iter().enumerate() {
        lines.push(format!("candidate {}:", index + 1));
        lines.push(format!("  syspath = {}", candidate.syspath.display()));
        lines.push(format!("  vid = 0x{:04X}", candidate.vendor_id));
        lines.push(format!("  pid = 0x{:04X}", candidate.product_id));
        lines.push(format!(
            "  mi = {}",
            format_optional_u8(candidate.interface_number)
        ));
        lines.push(format!(
            "  col = {}",
            format_optional_u8(candidate.collection_number)
        ));
        lines.push(format!(
            "  product = {}",
            candidate.product_name.as_deref().unwrap_or("unknown")
        ));
        lines.push(format!(
            "  serial = {}",
            candidate.serial_number.as_deref().unwrap_or("unknown")
        ));
        lines.push(format!(
            "  serial_prefix = {}",
            candidate.serial_prefix.summary()
        ));
        lines.push(format!(
            "  plausible_ms7e75 = {}",
            candidate.is_plausible_ms7e75()
        ));
        lines.push("  candidate_status = candidate only; HID support remains disabled".to_string());
    }

    lines.join("\n")
}

fn parse_candidate_metadata(path: &Path) -> Result<HidInventoryCandidate> {
    let uevent = read_uevent_map(path)?;
    let (vendor_id, product_id) = parse_vid_pid(&uevent, path)?;
    let serial_number = uevent
        .get("HID_UNIQ")
        .map(|serial| serial.trim().to_string())
        .filter(|serial| !serial.is_empty());

    Ok(HidInventoryCandidate {
        syspath: path.to_path_buf(),
        vendor_id,
        product_id,
        interface_number: parse_interface_number(path, &uevent)?,
        collection_number: parse_collection_number(path, &uevent)?,
        product_name: uevent
            .get("HID_NAME")
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty()),
        serial_prefix: parse_serial_prefix(serial_number.as_deref()),
        serial_number,
    })
}

fn matches_target_candidate(candidate: &HidInventoryCandidate) -> bool {
    candidate.vendor_id == TARGET_VENDOR_ID
        && candidate.product_id == TARGET_PRODUCT_ID
        && candidate
            .interface_number
            .is_none_or(|interface_number| interface_number == TARGET_INTERFACE_NUMBER)
        && candidate
            .collection_number
            .is_none_or(|collection_number| collection_number == TARGET_COLLECTION_NUMBER)
}

fn read_uevent_map(path: &Path) -> Result<BTreeMap<String, String>> {
    let uevent_path = path.join("uevent");
    let contents = fs::read_to_string(&uevent_path).map_err(|err| {
        Error::HidInventoryReadFailed(format!("unable to read {}: {err}", uevent_path.display()))
    })?;

    let mut map = BTreeMap::new();
    for line in contents.lines() {
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    Ok(map)
}

fn parse_vid_pid(uevent: &BTreeMap<String, String>, path: &Path) -> Result<(u16, u16)> {
    let hid_id = uevent.get("HID_ID").ok_or_else(|| {
        Error::HidInventoryReadFailed(format!("{} missing HID_ID in uevent", path.display()))
    })?;
    parse_hid_id(hid_id).ok_or_else(|| {
        Error::HidInventoryReadFailed(format!(
            "{} has invalid HID_ID value: {hid_id}",
            path.display()
        ))
    })
}

fn parse_hid_id(value: &str) -> Option<(u16, u16)> {
    let mut parts = value.trim().split(':');
    let _bus = parts.next()?;
    let vendor = parts.next()?;
    let product = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    let vendor_id = parse_hex_u16(vendor)?;
    let product_id = parse_hex_u16(product)?;
    Some((vendor_id, product_id))
}

fn parse_interface_number(path: &Path, uevent: &BTreeMap<String, String>) -> Result<Option<u8>> {
    if let Some(interface) = uevent.get("HID_INTERFACE") {
        return parse_optional_u8_field(interface, "HID_INTERFACE", path);
    }

    if let Some(interface) = read_optional_text_file(path.join("bInterfaceNumber"))? {
        return parse_optional_u8_field(&interface, "bInterfaceNumber", path);
    }

    if let Some(interface) = parse_interface_from_path(path) {
        return Ok(Some(interface));
    }

    Ok(None)
}

fn parse_collection_number(path: &Path, uevent: &BTreeMap<String, String>) -> Result<Option<u8>> {
    if let Some(collection) = uevent.get("HID_COLLECTION") {
        return parse_optional_u8_field(collection, "HID_COLLECTION", path);
    }

    if let Some(collection) = read_optional_text_file(path.join("collection"))? {
        return parse_optional_u8_field(&collection, "collection", path);
    }

    if let Some(collection) = read_optional_text_file(path.join("hid_collection"))? {
        return parse_optional_u8_field(&collection, "hid_collection", path);
    }

    Ok(parse_collection_from_path(path))
}

fn parse_optional_u8_field(value: &str, field: &str, path: &Path) -> Result<Option<u8>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    parse_hex_or_decimal_u8(trimmed).map(Some).ok_or_else(|| {
        Error::HidInventoryReadFailed(format!(
            "{} has invalid {} value: {}",
            path.display(),
            field,
            value.trim()
        ))
    })
}

fn parse_serial_prefix(serial: Option<&str>) -> SerialPrefixStatus {
    let Some(serial) = serial.map(str::trim).filter(|serial| !serial.is_empty()) else {
        return SerialPrefixStatus::Missing;
    };

    let Some(prefix) = serial.get(..4) else {
        return SerialPrefixStatus::Invalid(serial.to_string());
    };

    match u16::from_str_radix(prefix, 16) {
        Ok(board_id) if board_id == TARGET_BOARD_ID => {
            SerialPrefixStatus::ExpectedBoardId(board_id)
        }
        Ok(board_id) => SerialPrefixStatus::UnexpectedBoardId(board_id),
        Err(_) => SerialPrefixStatus::Invalid(serial.to_string()),
    }
}

fn read_optional_text_file(path: PathBuf) -> Result<Option<String>> {
    match fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(contents.trim().to_string())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(Error::HidInventoryReadFailed(format!(
            "unable to read {}: {err}",
            path.display()
        ))),
    }
}

fn parse_interface_from_path(path: &Path) -> Option<u8> {
    for component in path.components() {
        let component = component.as_os_str().to_string_lossy();
        if let Some((_, suffix)) = component.split_once(':')
            && let Some((_, interface)) = suffix.split_once('.')
            && let Some(parsed) = parse_hex_or_decimal_u8(interface)
        {
            return Some(parsed);
        }
    }

    None
}

fn parse_collection_from_path(path: &Path) -> Option<u8> {
    for component in path.components() {
        let component = component.as_os_str().to_string_lossy();
        if component.len() < 5 {
            continue;
        }

        let prefix = &component[..3];
        if prefix.eq_ignore_ascii_case("col")
            && let Some(parsed) = parse_hex_or_decimal_u8(&component[3..])
        {
            return Some(parsed);
        }
    }

    None
}

fn parse_hex_u16(value: &str) -> Option<u16> {
    let trimmed = value.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let tail = hex.get(hex.len().saturating_sub(4)..).unwrap_or(hex);
    u16::from_str_radix(tail, 16).ok()
}

fn parse_hex_or_decimal_u8(value: &str) -> Option<u8> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        return u8::from_str_radix(hex, 16).ok();
    }

    if trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) && trimmed.len() <= 2 {
        return u8::from_str_radix(trimmed, 16)
            .ok()
            .or_else(|| trimmed.parse::<u8>().ok());
    }

    trimmed.parse::<u8>().ok()
}

fn format_optional_u8(value: Option<u8>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        HidInventoryCandidate, SerialPrefixStatus, TARGET_PRODUCT_ID, TARGET_VENDOR_ID,
        format_inventory_report, inventory_candidates_from_root, parse_hid_id, parse_serial_prefix,
    };

    #[test]
    fn hid_id_parser_accepts_documented_vid_pid() {
        assert_eq!(
            parse_hid_id("0003:00000DB0:00000076"),
            Some((0x0DB0, 0x0076))
        );
        assert_eq!(parse_hid_id("0003:0DB0:0076"), Some((0x0DB0, 0x0076)));
    }

    #[test]
    fn serial_prefix_parser_accepts_expected_7e75_board_id() {
        assert_eq!(
            parse_serial_prefix(Some("7E75ABCD")),
            SerialPrefixStatus::ExpectedBoardId(0x7E75)
        );
    }

    #[test]
    fn serial_prefix_parser_reports_missing_and_invalid_values() {
        assert_eq!(parse_serial_prefix(None), SerialPrefixStatus::Missing);
        assert_eq!(parse_serial_prefix(Some("")), SerialPrefixStatus::Missing);
        assert_eq!(
            parse_serial_prefix(Some("XYZ123")),
            SerialPrefixStatus::Invalid("XYZ123".to_string())
        );
        assert_eq!(
            parse_serial_prefix(Some("12")),
            SerialPrefixStatus::Invalid("12".to_string())
        );
    }

    #[test]
    fn inventory_filters_matching_vid_pid_and_interface_collection() {
        let fixture = SysfsFixture::new();
        fixture.write_candidate(
            "candidate_1",
            &[
                ("HID_ID", "0003:00000DB0:00000076"),
                ("HID_NAME", "MSI Common HID"),
                ("HID_UNIQ", "7E75ABCD"),
                ("HID_INTERFACE", "0"),
                ("HID_COLLECTION", "0"),
            ],
        );
        fixture.write_candidate(
            "candidate_2",
            &[
                ("HID_ID", "0003:00000DB0:00000076"),
                ("HID_UNIQ", "7E75DCBA"),
                ("HID_INTERFACE", "1"),
            ],
        );
        fixture.write_candidate(
            "candidate_3",
            &[
                ("HID_ID", "0003:00000DB0:00000076"),
                ("HID_UNIQ", "7E75EEEE"),
                ("HID_COLLECTION", "1"),
            ],
        );

        let candidates = inventory_candidates_from_root(fixture.root()).unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].vendor_id, TARGET_VENDOR_ID);
        assert_eq!(candidates[0].product_id, TARGET_PRODUCT_ID);
        assert_eq!(candidates[0].interface_number, Some(0));
        assert_eq!(candidates[0].collection_number, Some(0));
    }

    #[test]
    fn inventory_uses_unknown_when_serial_is_missing() {
        let fixture = SysfsFixture::new();
        fixture.write_candidate(
            "candidate_missing_serial",
            &[("HID_ID", "0003:00000DB0:00000076")],
        );

        let candidates = inventory_candidates_from_root(fixture.root()).unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].serial_prefix, SerialPrefixStatus::Missing);
    }

    #[test]
    fn inventory_rejects_non_matching_vid_pid() {
        let fixture = SysfsFixture::new();
        fixture.write_candidate(
            "candidate_wrong_pid",
            &[
                ("HID_ID", "0003:00000DB0:00000075"),
                ("HID_UNIQ", "7E75ABCD"),
            ],
        );
        fixture.write_candidate(
            "candidate_wrong_vid",
            &[
                ("HID_ID", "0003:00001234:00000076"),
                ("HID_UNIQ", "7E75ABCD"),
            ],
        );

        let candidates = inventory_candidates_from_root(fixture.root()).unwrap();

        assert!(candidates.is_empty());
    }

    #[test]
    fn inventory_reports_invalid_serial_prefix_without_opening_devices() {
        let fixture = SysfsFixture::new();
        fixture.write_candidate(
            "candidate_invalid_serial",
            &[
                ("HID_ID", "0003:00000DB0:00000076"),
                ("HID_UNIQ", "GGGG9999"),
            ],
        );

        let candidates = inventory_candidates_from_root(fixture.root()).unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].serial_prefix,
            SerialPrefixStatus::Invalid("GGGG9999".to_string())
        );
    }

    #[test]
    fn inventory_report_is_explicitly_read_only_and_disabled() {
        let report = format_inventory_report(&[HidInventoryCandidate {
            syspath: PathBuf::from("/sys/bus/hid/devices/0003:0DB0:0076.0001"),
            vendor_id: 0x0DB0,
            product_id: 0x0076,
            interface_number: Some(0),
            collection_number: Some(0),
            product_name: Some("MSI Common HID".to_string()),
            serial_number: Some("7E75ABCD".to_string()),
            serial_prefix: SerialPrefixStatus::ExpectedBoardId(0x7E75),
        }]);

        assert!(report.contains("status = READ ONLY"));
        assert!(report.contains("mode = read-only metadata scan only"));
        assert!(report.contains("support = unsupported/not enabled"));
        assert!(report.contains("devices_opened = no"));
        assert!(report.contains("writes_enabled = no"));
        assert!(report.contains("writes_performed = no"));
        assert!(report.contains("next_safe_command = msi-ml linux hid gate"));
    }

    struct SysfsFixture {
        root: PathBuf,
    }

    impl SysfsFixture {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("msi_ml_hid_fixture_{unique}"));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn write_candidate(&self, dirname: &str, fields: &[(&str, &str)]) {
            let dir = self.root.join(dirname);
            fs::create_dir_all(&dir).unwrap();

            let mut uevent = String::new();
            for (key, value) in fields {
                uevent.push_str(key);
                uevent.push('=');
                uevent.push_str(value);
                uevent.push('\n');
            }

            fs::write(dir.join("uevent"), uevent).unwrap();
        }
    }

    impl Drop for SysfsFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
