use crate::linux::hid::gate::{HidBoardGateResult, HidGateStatus, read_hid_board_gate};
use crate::linux::hid::report::{
    Ms7e75Zone, ReportBuildError, RgbColor, build_zone_static_rgb_report,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DryRunError {
    InvalidColor(String),
    ReportBuild(ReportBuildError),
    GateRefused {
        status: HidGateStatus,
        reason: String,
    },
}

impl std::fmt::Display for DryRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidColor(color) => write!(
                f,
                "invalid color: {color} (expected exactly 6 hex characters like ff0000)"
            ),
            Self::ReportBuild(err) => write!(f, "{err}"),
            Self::GateRefused { status, reason } => {
                write!(
                    f,
                    "dry-run refused: status={} reason={reason}",
                    status.as_str()
                )
            }
        }
    }
}

impl std::error::Error for DryRunError {}

impl From<ReportBuildError> for DryRunError {
    fn from(value: ReportBuildError) -> Self {
        Self::ReportBuild(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DryRunFamily {
    Gen1,
    Gen2,
}

impl DryRunFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gen1 => "Gen1",
            Self::Gen2 => "Gen2",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunGateDecision {
    pub status: HidGateStatus,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidDryRunResult {
    pub zone: Ms7e75Zone,
    pub family: DryRunFamily,
    pub report_id: u8,
    pub report_length: usize,
    pub area_index: Option<u8>,
    pub port_index: Option<u8>,
    pub color: RgbColor,
    pub buffer: Vec<u8>,
    pub gate_status: HidGateStatus,
}

pub fn run_live_dry_run(zone: Ms7e75Zone, color: RgbColor) -> Result<HidDryRunResult, DryRunError> {
    let gate = read_hid_board_gate().map_err(|err| DryRunError::GateRefused {
        status: HidGateStatus::Blocked,
        reason: err.to_string(),
    })?;
    build_dry_run_report(zone, color, &gate, None)
}

pub fn build_dry_run_report(
    zone: Ms7e75Zone,
    color: RgbColor,
    gate: &HidBoardGateResult,
    fixture_override: Option<HidGateStatus>,
) -> Result<HidDryRunResult, DryRunError> {
    let gate_decision = evaluate_gate_for_dry_run(gate, fixture_override);
    if gate_decision.status != HidGateStatus::EligibleForDryRun {
        return Err(DryRunError::GateRefused {
            status: gate_decision.status,
            reason: gate_decision.reason,
        });
    }

    let buffer = build_zone_static_rgb_report(zone, color, false)?;
    let family = if zone.gen1_area_index().is_some() {
        DryRunFamily::Gen1
    } else {
        DryRunFamily::Gen2
    };

    Ok(HidDryRunResult {
        zone,
        family,
        report_id: zone.report_id(),
        report_length: buffer.len(),
        area_index: zone.gen1_area_index(),
        port_index: zone.gen2_port_index(),
        color,
        buffer,
        gate_status: HidGateStatus::EligibleForDryRun,
    })
}

pub fn evaluate_gate_for_dry_run(
    gate: &HidBoardGateResult,
    fixture_override: Option<HidGateStatus>,
) -> DryRunGateDecision {
    if let Some(status) = fixture_override {
        return DryRunGateDecision {
            status,
            reason: format!("explicit fixture/test gate override: {}", status.as_str()),
        };
    }

    DryRunGateDecision {
        status: gate.status,
        reason: format!(
            "Phase 2 gate status is {}; DMI={}, HID={}, serial={}",
            gate.status.as_str(),
            gate.dmi.summary,
            gate.inventory.summary,
            gate.serial.summary
        ),
    }
}

pub fn parse_rgb_hex(input: &str) -> Result<RgbColor, DryRunError> {
    let trimmed = input.trim();
    let hex = trimmed
        .strip_prefix('#')
        .or_else(|| trimmed.strip_prefix("0x"))
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(DryRunError::InvalidColor(input.to_string()));
    }

    let red = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| DryRunError::InvalidColor(input.to_string()))?;
    let green = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| DryRunError::InvalidColor(input.to_string()))?;
    let blue = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| DryRunError::InvalidColor(input.to_string()))?;

    Ok(RgbColor::new(red, green, blue))
}

pub fn format_dry_run_report(result: &HidDryRunResult) -> String {
    [
        "MS-7E75 HID dry-run".to_string(),
        format!("  zone = {}", result.zone),
        format!("  report_family = {}", result.family.as_str()),
        format!("  report_id = 0x{:02X}", result.report_id),
        format!("  report_length = {}", result.report_length),
        format!("  area_index = {}", format_optional_u8(result.area_index)),
        format!("  port_index = {}", format_optional_u8(result.port_index)),
        format!(
            "  color = {:02x}{:02x}{:02x}",
            result.color.red, result.color.green, result.color.blue
        ),
        format!("  hex_preview = {}", format_hex_preview(&result.buffer, 32)),
        "  status = DRY RUN ONLY".to_string(),
        "  devices_opened = no".to_string(),
        "  writes_performed = no".to_string(),
        "  message = no device opened, no writes performed".to_string(),
        "  support = unsupported/not enabled".to_string(),
    ]
    .join("\n")
}

fn format_hex_preview(buffer: &[u8], max_len: usize) -> String {
    let preview = buffer
        .iter()
        .take(max_len)
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ");

    if buffer.len() > max_len {
        format!("{preview} ...")
    } else {
        preview
    }
}

fn format_optional_u8(value: Option<u8>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "n/a".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::linux::hid::gate::{
        DmiGateResult, HidBoardGateResult, HidGateStatus, HidInventoryGateResult, SerialGateResult,
    };
    use crate::linux::hid::inventory::HidInventoryCandidate;
    use crate::linux::hid::report::Ms7e75Zone;

    use super::{DryRunError, build_dry_run_report, format_dry_run_report, parse_rgb_hex};

    #[test]
    fn dry_run_builds_jrgb1_report_for_valid_color() {
        let result = build_dry_run_report(
            Ms7e75Zone::Jrgb1,
            parse_rgb_hex("ff0000").unwrap(),
            &eligible_gate(),
            None,
        )
        .unwrap();

        assert_eq!(result.report_id, 0x50);
        assert_eq!(result.area_index, Some(9));
        assert_eq!(result.port_index, None);
    }

    #[test]
    fn dry_run_builds_gen2_reports_for_supported_ports() {
        let cases = [
            (Ms7e75Zone::JargbV2_1, 0x90, Some(0)),
            (Ms7e75Zone::JargbV2_2, 0x91, Some(1)),
            (Ms7e75Zone::JargbV2_3, 0x92, Some(2)),
            (Ms7e75Zone::EzConn, 0x93, Some(3)),
        ];

        for (zone, report_id, port_index) in cases {
            let result = build_dry_run_report(
                zone,
                parse_rgb_hex("00ff00").unwrap(),
                &eligible_gate(),
                None,
            )
            .unwrap();
            assert_eq!(result.report_id, report_id);
            assert_eq!(result.port_index, port_index);
        }
    }

    #[test]
    fn invalid_zone_is_rejected() {
        assert!(Ms7e75Zone::from_name("JARGB_V2_99").is_err());
    }

    #[test]
    fn invalid_color_is_rejected() {
        assert!(parse_rgb_hex("ff00").is_err());
        assert!(parse_rgb_hex("gg0000").is_err());
    }

    #[test]
    fn blocked_gate_refuses_dry_run() {
        let error = build_dry_run_report(
            Ms7e75Zone::Jrgb1,
            parse_rgb_hex("ff0000").unwrap(),
            &gate_with_status(HidGateStatus::Blocked),
            None,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            DryRunError::GateRefused {
                status: HidGateStatus::Blocked,
                ..
            }
        ));
    }

    #[test]
    fn inconclusive_gate_refuses_dry_run() {
        let error = build_dry_run_report(
            Ms7e75Zone::Jrgb1,
            parse_rgb_hex("ff0000").unwrap(),
            &gate_with_status(HidGateStatus::Inconclusive),
            None,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            DryRunError::GateRefused {
                status: HidGateStatus::Inconclusive,
                ..
            }
        ));
    }

    #[test]
    fn eligible_gate_allows_dry_run_buffer_generation() {
        let result = build_dry_run_report(
            Ms7e75Zone::JargbV2_1,
            parse_rgb_hex("112233").unwrap(),
            &eligible_gate(),
            None,
        )
        .unwrap();

        assert_eq!(result.buffer[0], 0x90);
        assert_eq!(result.buffer.len(), 302);
    }

    #[test]
    fn formatted_output_contains_dry_run_only_no_writes_wording() {
        let report = format_dry_run_report(
            &build_dry_run_report(
                Ms7e75Zone::Jrgb1,
                parse_rgb_hex("ff0000").unwrap(),
                &eligible_gate(),
                None,
            )
            .unwrap(),
        );

        assert!(report.contains("DRY RUN ONLY"));
        assert!(report.contains("devices_opened = no"));
        assert!(report.contains("writes_performed = no"));
        assert!(report.contains("no device opened, no writes performed"));
    }

    #[test]
    fn fixture_override_can_bypass_live_gate_for_tests() {
        let result = build_dry_run_report(
            Ms7e75Zone::Jrgb1,
            parse_rgb_hex("ff0000").unwrap(),
            &gate_with_status(HidGateStatus::Blocked),
            Some(HidGateStatus::EligibleForDryRun),
        )
        .unwrap();

        assert_eq!(result.report_id, 0x50);
    }

    fn eligible_gate() -> HidBoardGateResult {
        gate_with_status(HidGateStatus::EligibleForDryRun)
    }

    fn gate_with_status(status: HidGateStatus) -> HidBoardGateResult {
        HidBoardGateResult {
            status,
            dmi: DmiGateResult {
                matched: status == HidGateStatus::EligibleForDryRun,
                summary: "test dmi".to_string(),
            },
            inventory: HidInventoryGateResult {
                matched: status == HidGateStatus::EligibleForDryRun,
                summary: "test inventory".to_string(),
                candidate_count: 1,
            },
            serial: SerialGateResult {
                status,
                summary: "test serial".to_string(),
            },
        }
    }

    #[test]
    fn no_device_access_types_are_needed_for_dry_run_tests() {
        let candidate = HidInventoryCandidate {
            syspath: PathBuf::from("/sys/bus/hid/devices/test"),
            vendor_id: 0x0DB0,
            product_id: 0x0076,
            interface_number: Some(0),
            collection_number: Some(0),
            product_name: Some("MSI Common HID".to_string()),
            serial_number: Some("7E75ABCD".to_string()),
            serial_prefix: crate::linux::hid::inventory::SerialPrefixStatus::ExpectedBoardId(
                0x7E75,
            ),
        };

        assert_eq!(candidate.vendor_id, 0x0DB0);
    }
}
