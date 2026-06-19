use crate::linux::hid::capture_compare::{
    LIVE_0X50_SETUP_BYTES, LiveJargbV2_1Preset, build_live_confirmed_jargb_v2_1_0x50_payload,
    extract_live_fixture_payload, first_difference_offset,
};
use crate::linux::hid::report::{GEN1_REPORT_ID, Ms7e75Zone, RgbColor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LivePayloadDryRunError {
    UnsupportedZone(String),
    UnsupportedMode(String),
    UnsupportedColor {
        mode: String,
        color: String,
    },
    FixtureDecode(String),
    FixtureMismatch {
        preset: LiveJargbV2_1Preset,
        first_difference_offset: usize,
    },
}

impl std::fmt::Display for LivePayloadDryRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedZone(zone) => write!(
                f,
                "unsupported zone for exact live payload dry-run: {zone} (expected JARGB_V2_1)"
            ),
            Self::UnsupportedMode(mode) => write!(
                f,
                "unsupported mode for exact live payload dry-run: {mode} (expected steady, breath, or off)"
            ),
            Self::UnsupportedColor { mode, color } => write!(
                f,
                "unsupported color for exact live payload dry-run: mode={mode} color={color}"
            ),
            Self::FixtureDecode(error) => {
                write!(f, "failed to decode checked-in live fixture: {error}")
            }
            Self::FixtureMismatch {
                preset,
                first_difference_offset,
            } => write!(
                f,
                "exact live payload dry-run fixture mismatch for {:?} at payload offset {}",
                preset, first_difference_offset
            ),
        }
    }
}

impl std::error::Error for LivePayloadDryRunError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivePayloadMode {
    Steady,
    Breath,
    Off,
}

impl LivePayloadMode {
    pub fn parse(input: &str) -> Result<Self, LivePayloadDryRunError> {
        match input {
            "steady" => Ok(Self::Steady),
            "breath" => Ok(Self::Breath),
            "off" => Ok(Self::Off),
            _ => Err(LivePayloadDryRunError::UnsupportedMode(input.to_string())),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Steady => "steady",
            Self::Breath => "breath",
            Self::Off => "off",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactLivePayloadDryRunResult {
    pub board_profile: &'static str,
    pub zone: Ms7e75Zone,
    pub mode: LivePayloadMode,
    pub color: RgbColor,
    pub preset: LiveJargbV2_1Preset,
    pub setup_bytes: [u8; 8],
    pub report_id: u8,
    pub payload: Vec<u8>,
}

pub fn build_exact_live_payload_dry_run(
    zone: Ms7e75Zone,
    mode: LivePayloadMode,
    color: RgbColor,
) -> Result<ExactLivePayloadDryRunResult, LivePayloadDryRunError> {
    if zone != Ms7e75Zone::JargbV2_1 {
        return Err(LivePayloadDryRunError::UnsupportedZone(zone.to_string()));
    }

    let preset = preset_for(mode, color)?;
    let payload = build_live_confirmed_jargb_v2_1_0x50_payload(preset);
    let fixture = extract_live_fixture_payload(preset)
        .map_err(|error| LivePayloadDryRunError::FixtureDecode(error.to_string()))?;

    let first_diff = first_difference_offset(&payload, &fixture.payload).or_else(|| {
        if payload.len() == fixture.payload.len() {
            None
        } else {
            Some(payload.len().min(fixture.payload.len()))
        }
    });
    if let Some(first_difference_offset) = first_diff {
        return Err(LivePayloadDryRunError::FixtureMismatch {
            preset,
            first_difference_offset,
        });
    }

    Ok(ExactLivePayloadDryRunResult {
        board_profile: "MS-7E75",
        zone,
        mode,
        color,
        preset,
        setup_bytes: LIVE_0X50_SETUP_BYTES,
        report_id: GEN1_REPORT_ID,
        payload: payload.to_vec(),
    })
}

pub fn format_exact_live_payload_dry_run_report(result: &ExactLivePayloadDryRunResult) -> String {
    [
        "MS-7E75 exact live payload dry-run".to_string(),
        format!("  board_profile = {}", result.board_profile),
        format!("  zone = {}", result.zone),
        format!("  mode = {}", result.mode.as_str()),
        format!(
            "  rgb = {:02x}{:02x}{:02x}",
            result.color.red, result.color.green, result.color.blue
        ),
        "  status = DRY RUN ONLY".to_string(),
        "  devices_opened = no".to_string(),
        "  writes_enabled = no".to_string(),
        "  writes_performed = no".to_string(),
        format!("  setup_bytes = {}", format_hex_line(&result.setup_bytes)),
        format!("  report_id = 0x{:02X}", result.report_id),
        format!("  payload_len = {}", result.payload.len()),
        "  fixture_match = yes".to_string(),
        "  payload_hex_dump =".to_string(),
        format_hex_dump(&result.payload),
        "  message = OFFLINE ONLY exact checked-in MSI Center payload; no device opened, no write performed".to_string(),
        "  support = unsupported/not enabled".to_string(),
    ]
    .join("\n")
}

fn preset_for(
    mode: LivePayloadMode,
    color: RgbColor,
) -> Result<LiveJargbV2_1Preset, LivePayloadDryRunError> {
    let hex = format!("{:02x}{:02x}{:02x}", color.red, color.green, color.blue);

    match (mode, color.red, color.green, color.blue) {
        (LivePayloadMode::Steady, 0xFF, 0x00, 0x00) => Ok(LiveJargbV2_1Preset::SteadyRed),
        (LivePayloadMode::Steady, 0x00, 0xFF, 0x00) => Ok(LiveJargbV2_1Preset::SteadyGreen),
        (LivePayloadMode::Steady, 0x00, 0x00, 0xFF) => Ok(LiveJargbV2_1Preset::SteadyBlue),
        (LivePayloadMode::Breath, 0xFF, 0x00, 0x00) => Ok(LiveJargbV2_1Preset::BreathRed),
        (LivePayloadMode::Off, 0xFF, 0x00, 0x00) => Ok(LiveJargbV2_1Preset::OffRetainedRed),
        _ => Err(LivePayloadDryRunError::UnsupportedColor {
            mode: mode.as_str().to_string(),
            color: hex,
        }),
    }
}

fn format_hex_line(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_hex_dump(bytes: &[u8]) -> String {
    bytes
        .chunks(16)
        .enumerate()
        .map(|(index, chunk)| format!("    {:04X}: {}", index * 16, format_hex_line(chunk)))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::linux::hid::report::Ms7e75Zone;

    use super::{
        LivePayloadDryRunError, LivePayloadMode, build_exact_live_payload_dry_run,
        format_exact_live_payload_dry_run_report,
    };

    #[test]
    fn exact_live_payload_dry_run_supports_all_confirmed_cases() {
        let cases = [
            (
                LivePayloadMode::Steady,
                "ff0000",
                "mode = steady",
                "rgb = ff0000",
            ),
            (
                LivePayloadMode::Steady,
                "00ff00",
                "mode = steady",
                "rgb = 00ff00",
            ),
            (
                LivePayloadMode::Steady,
                "0000ff",
                "mode = steady",
                "rgb = 0000ff",
            ),
            (
                LivePayloadMode::Breath,
                "ff0000",
                "mode = breath",
                "rgb = ff0000",
            ),
            (LivePayloadMode::Off, "ff0000", "mode = off", "rgb = ff0000"),
        ];

        for (mode, color_hex, expected_mode_line, expected_rgb_line) in cases {
            let color = super::super::dry_run::parse_rgb_hex(color_hex).unwrap();
            let result =
                build_exact_live_payload_dry_run(Ms7e75Zone::JargbV2_1, mode, color).unwrap();
            let report = format_exact_live_payload_dry_run_report(&result);

            assert!(report.contains("MS-7E75"));
            assert!(report.contains("zone = JARGB_V2_1"));
            assert!(report.contains(expected_mode_line));
            assert!(report.contains(expected_rgb_line));
            assert!(report.contains("status = DRY RUN ONLY"));
            assert!(report.contains("devices_opened = no"));
            assert!(report.contains("writes_enabled = no"));
            assert!(report.contains("writes_performed = no"));
            assert!(report.contains("setup_bytes = 21 09 50 03 00 00 22 01"));
            assert!(report.contains("report_id = 0x50"));
            assert!(report.contains("payload_len = 290"));
            assert!(report.contains("fixture_match = yes"));
            assert!(report.contains("0000: 50"));
            assert!(report.contains("0120:"));
        }
    }

    #[test]
    fn unsupported_zone_is_rejected_without_fallback() {
        let error = build_exact_live_payload_dry_run(
            Ms7e75Zone::JargbV2_2,
            LivePayloadMode::Steady,
            super::super::dry_run::parse_rgb_hex("ff0000").unwrap(),
        )
        .unwrap_err();

        assert!(matches!(error, LivePayloadDryRunError::UnsupportedZone(_)));
    }

    #[test]
    fn unsupported_color_is_rejected_without_fallback() {
        let error = build_exact_live_payload_dry_run(
            Ms7e75Zone::JargbV2_1,
            LivePayloadMode::Breath,
            super::super::dry_run::parse_rgb_hex("00ff00").unwrap(),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            LivePayloadDryRunError::UnsupportedColor { .. }
        ));
    }

    #[test]
    fn unsupported_mode_string_is_rejected() {
        let error = LivePayloadMode::parse("wave").unwrap_err();
        assert!(matches!(error, LivePayloadDryRunError::UnsupportedMode(_)));
    }
}
