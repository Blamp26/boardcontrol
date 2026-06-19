#![allow(dead_code)]

use crate::linux::hid::report::{GEN1_REPORT_ID, GEN1_REPORT_LENGTH};

const USB_SETUP_LEN: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureCompareError {
    InvalidHexToken(String),
    EmptyInput,
    MissingPayload,
    UnexpectedReportId { expected: u8, actual: u8 },
}

impl std::fmt::Display for CaptureCompareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHexToken(token) => write!(f, "invalid hex byte token: {token}"),
            Self::EmptyInput => write!(f, "capture fixture is empty"),
            Self::MissingPayload => write!(f, "capture fixture does not contain a HID payload"),
            Self::UnexpectedReportId { expected, actual } => write!(
                f,
                "unexpected report ID 0x{actual:02X}, expected 0x{expected:02X}"
            ),
        }
    }
}

impl std::error::Error for CaptureCompareError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbSetupPacket {
    pub bm_request_type: u8,
    pub b_request: u8,
    pub w_value: u16,
    pub w_index: u16,
    pub w_length: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportShape {
    pub report_id: u8,
    pub report_length: u16,
}

impl UsbSetupPacket {
    pub const fn report_id(self) -> u8 {
        (self.w_value & 0x00FF) as u8
    }

    pub const fn report_type(self) -> u8 {
        (self.w_value >> 8) as u8
    }

    pub const fn is_msi_center_0x50_setup(self) -> bool {
        self.bm_request_type == 0x21
            && self.b_request == 0x09
            && self.w_value == 0x0350
            && self.w_length == GEN1_REPORT_LENGTH as u16
    }

    pub const fn report_shape(self) -> ReportShape {
        ReportShape {
            report_id: self.report_id(),
            report_length: self.w_length,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedHidPayload {
    pub setup: Option<UsbSetupPacket>,
    pub setup_offset: Option<usize>,
    pub payload_offset: usize,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedReportSummary {
    pub frame: u32,
    pub setup: UsbSetupPacket,
    pub payload_prefix: Vec<u8>,
    pub report_length: usize,
    pub store_byte: u8,
}

impl CapturedReportSummary {
    pub fn is_live_confirmed_0x50_report(&self) -> bool {
        self.setup.is_msi_center_0x50_setup()
            && self.report_length == GEN1_REPORT_LENGTH
            && self.payload_prefix.first().copied() == Some(GEN1_REPORT_ID)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveJargbV2_1Preset {
    SteadyRed,
    SteadyGreen,
    SteadyBlue,
    BreathRed,
    OffRetainedRed,
}

impl LiveJargbV2_1Preset {
    pub const fn mode(self) -> u8 {
        match self {
            Self::SteadyRed | Self::SteadyGreen | Self::SteadyBlue => 0x02,
            Self::BreathRed => 0x04,
            Self::OffRetainedRed => 0x00,
        }
    }

    pub const fn rgb(self) -> [u8; 3] {
        match self {
            Self::SteadyRed | Self::BreathRed | Self::OffRetainedRed => [0xFF, 0x00, 0x00],
            Self::SteadyGreen => [0x00, 0xFF, 0x00],
            Self::SteadyBlue => [0x00, 0x00, 0xFF],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteComparisonStatus {
    Match,
    Differs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteComparison {
    pub offset: usize,
    pub captured: u8,
    pub builder: u8,
    pub status: ByteComparisonStatus,
    pub meaning: String,
}

pub fn first_difference_offset(lhs: &[u8], rhs: &[u8]) -> Option<usize> {
    let shared_len = lhs.len().min(rhs.len());

    for offset in 0..shared_len {
        if lhs[offset] != rhs[offset] {
            return Some(offset);
        }
    }

    if lhs.len() == rhs.len() {
        None
    } else {
        Some(shared_len)
    }
}

pub fn parse_hex_fixture(input: &str) -> Result<Vec<u8>, CaptureCompareError> {
    let mut bytes = Vec::new();

    for raw_token in input.split_whitespace() {
        let token = raw_token
            .trim_matches(|ch: char| ch == ',' || ch == ';' || ch == ':')
            .trim_start_matches("0x")
            .trim_start_matches("0X");

        if token.is_empty() {
            continue;
        }

        let byte = u8::from_str_radix(token, 16)
            .map_err(|_| CaptureCompareError::InvalidHexToken(raw_token.to_string()))?;
        bytes.push(byte);
    }

    if bytes.is_empty() {
        return Err(CaptureCompareError::EmptyInput);
    }

    Ok(bytes)
}

pub fn parse_usbpcap_hexdump_fixture(input: &str) -> Result<Vec<u8>, CaptureCompareError> {
    let mut normalized = Vec::new();

    for line in input.lines() {
        let mut parts = line.split_whitespace();
        if let Some(first) = parts.next() {
            let is_offset = first.len() == 4 && first.chars().all(|ch| ch.is_ascii_hexdigit());
            if !is_offset {
                normalized.push(first);
            }
        }
        normalized.extend(parts);
    }

    parse_hex_fixture(&normalized.join(" "))
}

pub fn extract_0x50_payload(bytes: &[u8]) -> Result<ExtractedHidPayload, CaptureCompareError> {
    if bytes.is_empty() {
        return Err(CaptureCompareError::EmptyInput);
    }

    if bytes[0] == GEN1_REPORT_ID {
        return Ok(ExtractedHidPayload {
            setup: None,
            setup_offset: None,
            payload_offset: 0,
            payload: bytes.to_vec(),
        });
    }

    for offset in 0..bytes.len().saturating_sub(USB_SETUP_LEN) {
        let setup = decode_setup(&bytes[offset..offset + USB_SETUP_LEN]);
        let payload_offset = offset + USB_SETUP_LEN;

        if setup.is_msi_center_0x50_setup()
            && bytes.get(payload_offset).copied() == Some(GEN1_REPORT_ID)
        {
            return Ok(ExtractedHidPayload {
                setup: Some(setup),
                setup_offset: Some(offset),
                payload_offset,
                payload: bytes[payload_offset..].to_vec(),
            });
        }
    }

    Err(CaptureCompareError::MissingPayload)
}

pub fn find_usb_setup_packets(bytes: &[u8]) -> Vec<UsbSetupPacket> {
    if bytes.len() < USB_SETUP_LEN {
        return Vec::new();
    }

    (0..=bytes.len() - USB_SETUP_LEN)
        .map(|offset| decode_setup(&bytes[offset..offset + USB_SETUP_LEN]))
        .filter(|setup| setup.bm_request_type == 0x21 && setup.b_request == 0x09)
        .collect()
}

pub fn contains_report_shape(bytes: &[u8], shape: ReportShape) -> bool {
    find_usb_setup_packets(bytes)
        .iter()
        .any(|setup| setup.report_shape() == shape)
}

pub fn compare_0x50_payload_to_gen1_builder(
    captured_payload: &[u8],
    builder_payload: &[u8],
) -> Result<Vec<ByteComparison>, CaptureCompareError> {
    if captured_payload.is_empty() {
        return Err(CaptureCompareError::EmptyInput);
    }

    if captured_payload[0] != GEN1_REPORT_ID {
        return Err(CaptureCompareError::UnexpectedReportId {
            expected: GEN1_REPORT_ID,
            actual: captured_payload[0],
        });
    }

    let compared_len = captured_payload.len().min(builder_payload.len());
    let comparisons = (0..compared_len)
        .map(|offset| {
            let captured = captured_payload[offset];
            let builder = builder_payload[offset];
            let status = if captured == builder {
                ByteComparisonStatus::Match
            } else {
                ByteComparisonStatus::Differs
            };

            ByteComparison {
                offset,
                captured,
                builder,
                status,
                meaning: describe_gen1_offset(offset),
            }
        })
        .collect();

    Ok(comparisons)
}

pub fn build_live_confirmed_jargb_v2_1_0x50_payload(
    preset: LiveJargbV2_1Preset,
) -> [u8; GEN1_REPORT_LENGTH] {
    let mut payload = [0_u8; GEN1_REPORT_LENGTH];
    let mode = preset.mode();
    payload[0] = GEN1_REPORT_ID;
    payload[1] = mode;
    let primary = preset.rgb();

    write_live_record(
        &mut payload,
        0,
        mode,
        [
            primary,
            [0x00, 0xFF, 0x00],
            [0x00, 0x00, 0xFF],
            [0xFF, 0xFF, 0xFF],
        ],
        0x00,
        0x35,
        0x1E,
    );

    for record_index in 1..=3 {
        write_live_record(
            &mut payload,
            record_index,
            0x01,
            [
                primary,
                [0x00, 0xFF, 0x00],
                [0x00, 0x00, 0xFF],
                [0xFF, 0xFF, 0xFF],
            ],
            0x03,
            0x94,
            0x1E,
        );
    }

    for record_index in [4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16] {
        write_live_record(
            &mut payload,
            record_index,
            0x02,
            [
                [0xFF, 0x00, 0x00],
                [0xFF, 0x00, 0x00],
                [0xFF, 0x00, 0x00],
                [0xFF, 0x00, 0x00],
            ],
            0x00,
            0xB5,
            0x1E,
        );
    }

    write_live_record(
        &mut payload,
        9,
        0x01,
        [
            [0x00, 0x00, 0x00],
            [0x00, 0x00, 0x00],
            [0x00, 0x00, 0x00],
            [0x00, 0x00, 0x00],
        ],
        0x03,
        0x94,
        0xFF,
    );

    write_live_record(
        &mut payload,
        17,
        0x02,
        [
            [0xFF, 0xFF, 0xFF],
            [0x00, 0xFF, 0x00],
            [0x00, 0x00, 0xFF],
            [0xFF, 0xFF, 0xFF],
        ],
        0x00,
        0xB5,
        0xFF,
    );

    payload[GEN1_REPORT_LENGTH - 1] = 0x01;
    payload
}

fn decode_setup(bytes: &[u8]) -> UsbSetupPacket {
    UsbSetupPacket {
        bm_request_type: bytes[0],
        b_request: bytes[1],
        w_value: u16::from_le_bytes([bytes[2], bytes[3]]),
        w_index: u16::from_le_bytes([bytes[4], bytes[5]]),
        w_length: u16::from_le_bytes([bytes[6], bytes[7]]),
    }
}

fn describe_gen1_offset(offset: usize) -> String {
    if offset == 0 {
        return "report ID".to_string();
    }

    if offset == GEN1_REPORT_LENGTH - 1 {
        return "store flag".to_string();
    }

    let zero_based = offset - 1;
    let area = zero_based / 16;
    let field = zero_based % 16;
    let field_name = match field {
        0 => "mode",
        1..=3 => "color 1 RGB",
        4..=6 => "color 2 RGB",
        7..=9 => "color 3 RGB",
        10..=12 => "color 4 RGB",
        13 => "color count option",
        14 => "packed option byte",
        15 => "cycle number",
        _ => unreachable!("modulo 16 field is always 0..=15"),
    };

    format!("Gen1 area {area} {field_name}")
}

fn write_live_record(
    payload: &mut [u8; GEN1_REPORT_LENGTH],
    record_index: usize,
    mode: u8,
    colors: [[u8; 3]; 4],
    option_1: u8,
    option_2: u8,
    cycle: u8,
) {
    let base = 1 + record_index * 16;
    payload[base] = mode;
    for (color_index, [red, green, blue]) in colors.into_iter().enumerate() {
        let offset = base + 1 + color_index * 3;
        payload[offset] = red;
        payload[offset + 1] = green;
        payload[offset + 2] = blue;
    }
    payload[base + 13] = option_1;
    payload[base + 14] = option_2;
    payload[base + 15] = cycle;
}

#[cfg(test)]
mod tests {
    use crate::linux::hid::report::{
        GEN1_REPORT_ID, GEN1_REPORT_LENGTH, Ms7e75Zone, RgbColor, build_zone_static_rgb_report,
    };

    use super::{
        ByteComparisonStatus, CapturedReportSummary, LiveJargbV2_1Preset, ReportShape,
        build_live_confirmed_jargb_v2_1_0x50_payload, compare_0x50_payload_to_gen1_builder,
        contains_report_shape, extract_0x50_payload, first_difference_offset, parse_hex_fixture,
        parse_usbpcap_hexdump_fixture,
    };

    const SETUP_0X50_290: &str = "21 09 50 03 00 00 22 01";
    const FRAME_4781_PREFIX: &str = "50 02 14 ff 09 00 ff 00 00 00 ff ff ff ff 00 35 1e";
    const FRAME_7757_PREFIX: &str = "50 03 ff 00 00 ff 64 00 00 00 ff ff ff ff 01 35 1e";
    const FRAME_4781_WITH_SETUP: &str =
        "21 09 50 03 00 00 22 01 50 02 14 ff 09 00 ff 00 00 00 ff ff ff ff 00 35 1e";
    const FIXTURE_STREAM_WITH_LIVE_SETUPS: &str = "\
        21 09 50 03 00 00 22 01 \
        50 02 14 ff 09 00 ff 00 00 00 ff ff ff ff 00 35 1e \
        21 09 50 03 00 00 22 01 \
        50 03 ff 00 00 ff 64 00 00 00 ff ff ff ff 01 35 1e";
    struct LiveModeFixture {
        label: &'static str,
        bytes: &'static str,
        store_byte: u8,
        preset: LiveJargbV2_1Preset,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct LiveModePreview {
        mode: u8,
        rgb: [u8; 3],
    }

    const LIVE_MODE_FIXTURES: [LiveModeFixture; 5] = [
        LiveModeFixture {
            label: "steady_red",
            bytes: include_str!("capture_compare_test2_fixture.txt"),
            store_byte: 0x01,
            preset: LiveJargbV2_1Preset::SteadyRed,
        },
        LiveModeFixture {
            label: "steady_green",
            bytes: include_str!("capture_compare_test3_fixture.txt"),
            store_byte: 0x01,
            preset: LiveJargbV2_1Preset::SteadyGreen,
        },
        LiveModeFixture {
            label: "steady_blue",
            bytes: include_str!("capture_compare_test4_fixture.txt"),
            store_byte: 0x01,
            preset: LiveJargbV2_1Preset::SteadyBlue,
        },
        LiveModeFixture {
            label: "breath_red",
            bytes: include_str!("capture_compare_test5_fixture.txt"),
            store_byte: 0x01,
            preset: LiveJargbV2_1Preset::BreathRed,
        },
        LiveModeFixture {
            label: "off_red_rgb_retained",
            bytes: include_str!("capture_compare_test6_fixture.txt"),
            store_byte: 0x01,
            preset: LiveJargbV2_1Preset::OffRetainedRed,
        },
    ];

    #[test]
    fn setup_prefixed_fixture_extracts_payload_after_usb_setup() {
        let bytes = parse_hex_fixture(FRAME_4781_WITH_SETUP).unwrap();
        let extracted = extract_0x50_payload(&bytes).unwrap();
        let setup = extracted.setup.unwrap();

        assert_eq!(extracted.setup_offset, Some(0));
        assert_eq!(extracted.payload_offset, 8);
        assert_eq!(
            extracted.payload,
            parse_hex_fixture(FRAME_4781_PREFIX).unwrap()
        );
        assert_eq!(setup.bm_request_type, 0x21);
        assert_eq!(setup.b_request, 0x09);
        assert_eq!(setup.w_value, 0x0350);
        assert_eq!(setup.report_type(), 0x03);
        assert_eq!(setup.report_id(), 0x50);
        assert_eq!(setup.w_length, GEN1_REPORT_LENGTH as u16);
        assert!(setup.is_msi_center_0x50_setup());
    }

    #[test]
    fn pcap_derived_setup_sequence_decodes_0x0350_and_length_290() {
        let bytes = parse_hex_fixture(SETUP_0X50_290).unwrap();
        let setups = super::find_usb_setup_packets(&bytes);

        assert_eq!(setups.len(), 1);
        assert_eq!(setups[0].bm_request_type, 0x21);
        assert_eq!(setups[0].b_request, 0x09);
        assert_eq!(setups[0].w_value, 0x0350);
        assert_eq!(setups[0].report_id(), 0x50);
        assert_eq!(setups[0].report_type(), 0x03);
        assert_eq!(setups[0].w_length, 290);
        assert_eq!(
            setups[0].report_shape(),
            ReportShape {
                report_id: 0x50,
                report_length: 290
            }
        );
    }

    #[test]
    fn direct_payload_fixture_needs_no_setup_header() {
        let bytes = parse_hex_fixture(FRAME_7757_PREFIX).unwrap();
        let extracted = extract_0x50_payload(&bytes).unwrap();

        assert_eq!(extracted.setup, None);
        assert_eq!(extracted.payload_offset, 0);
        assert_eq!(extracted.payload, bytes);
    }

    #[test]
    fn frame_4781_prefix_matches_report_id_and_differs_in_area0_fields() {
        let captured = parse_hex_fixture(FRAME_4781_PREFIX).unwrap();
        let builder =
            build_zone_static_rgb_report(Ms7e75Zone::Jrgb1, RgbColor::new(0xFF, 0, 0), false)
                .unwrap();
        let comparisons = compare_0x50_payload_to_gen1_builder(&captured, &builder).unwrap();

        assert_eq!(comparisons[0].status, ByteComparisonStatus::Match);
        assert_eq!(comparisons[0].meaning, "report ID");

        let differing_offsets = differing_offsets(&comparisons);
        assert_eq!(
            differing_offsets,
            vec![1, 2, 3, 4, 6, 10, 11, 12, 13, 15, 16]
        );
        assert_eq!(comparisons[1].meaning, "Gen1 area 0 mode");
        assert_eq!(comparisons[2].meaning, "Gen1 area 0 color 1 RGB");
        assert_eq!(comparisons[6].meaning, "Gen1 area 0 color 2 RGB");
    }

    #[test]
    fn frame_7757_prefix_matches_report_id_and_differs_in_area0_fields() {
        let captured = parse_hex_fixture(FRAME_7757_PREFIX).unwrap();
        let builder =
            build_zone_static_rgb_report(Ms7e75Zone::Jrgb1, RgbColor::new(0xFF, 0, 0), false)
                .unwrap();
        let comparisons = compare_0x50_payload_to_gen1_builder(&captured, &builder).unwrap();

        assert_eq!(comparisons[0].status, ByteComparisonStatus::Match);
        assert_eq!(comparisons[0].meaning, "report ID");

        let differing_offsets = differing_offsets(&comparisons);
        assert_eq!(
            differing_offsets,
            vec![1, 2, 5, 6, 10, 11, 12, 13, 14, 15, 16]
        );
        assert_eq!(comparisons[1].meaning, "Gen1 area 0 mode");
        assert_eq!(comparisons[5].meaning, "Gen1 area 0 color 2 RGB");
    }

    #[test]
    fn non_0x50_payload_is_rejected_before_comparison() {
        let builder =
            build_zone_static_rgb_report(Ms7e75Zone::Jrgb1, RgbColor::new(0xFF, 0, 0), false)
                .unwrap();
        let error = compare_0x50_payload_to_gen1_builder(&[0x90, 0x00], &builder).unwrap_err();

        assert!(error.to_string().contains("expected 0x50"));
    }

    #[test]
    fn live_mode_fixtures_all_decode_as_0x50_290_reports() {
        for fixture in LIVE_MODE_FIXTURES {
            let extracted =
                extract_0x50_payload(&parse_usbpcap_hexdump_fixture(fixture.bytes).unwrap())
                    .unwrap();
            let setup = extracted
                .setup
                .unwrap_or_else(|| panic!("fixture {} missing USB setup", fixture.label));

            assert_eq!(
                setup.report_shape(),
                ReportShape {
                    report_id: 0x50,
                    report_length: 290
                }
            );
            assert!(
                setup.is_msi_center_0x50_setup(),
                "fixture {}",
                fixture.label
            );
            assert_eq!(
                extracted.setup_offset,
                Some(28),
                "fixture {}",
                fixture.label
            );
            assert_eq!(
                extracted.payload.len(),
                GEN1_REPORT_LENGTH,
                "fixture {}",
                fixture.label
            );
            assert_eq!(extracted.payload[0], 0x50, "fixture {}", fixture.label);
            assert_eq!(
                extracted.payload[GEN1_REPORT_LENGTH - 1],
                0x01,
                "fixture {}",
                fixture.label
            );
            assert_eq!(fixture.store_byte, 0x01, "fixture {}", fixture.label);
        }
    }

    #[test]
    fn full_live_mode_fixtures_now_extract_complete_290_byte_payloads() {
        for fixture in LIVE_MODE_FIXTURES {
            let extracted =
                extract_0x50_payload(&parse_usbpcap_hexdump_fixture(fixture.bytes).unwrap())
                    .unwrap();

            assert_eq!(
                extracted.payload.len(),
                GEN1_REPORT_LENGTH,
                "fixture {}",
                fixture.label
            );
        }
    }

    #[test]
    fn offline_live_builder_matches_full_captured_payloads_byte_for_byte() {
        for fixture in LIVE_MODE_FIXTURES {
            let extracted =
                extract_0x50_payload(&parse_usbpcap_hexdump_fixture(fixture.bytes).unwrap())
                    .unwrap();
            let expected_builder = build_live_confirmed_jargb_v2_1_0x50_payload(fixture.preset);

            assert_eq!(extracted.payload.len(), GEN1_REPORT_LENGTH);
            assert_eq!(expected_builder.len(), GEN1_REPORT_LENGTH);
            assert_eq!(
                first_difference_offset(&extracted.payload, &expected_builder),
                None,
                "fixture {} differs at offset {:?}",
                fixture.label,
                first_difference_offset(&extracted.payload, &expected_builder)
            );
            assert_eq!(
                extracted.payload,
                expected_builder.to_vec(),
                "fixture {} first differing offset {:?}",
                fixture.label,
                first_difference_offset(&extracted.payload, &expected_builder)
            );
        }
    }

    #[test]
    fn offline_live_builder_matches_steady_red_fixture_prefix_and_store_metadata() {
        assert_live_builder_prefix(
            LiveJargbV2_1Preset::SteadyRed,
            &extract_fixture_prefix("steady_red"),
            0x02,
            [0xFF, 0x00, 0x00],
        );
    }

    #[test]
    fn offline_live_builder_matches_steady_green_fixture_prefix_and_store_metadata() {
        assert_live_builder_prefix(
            LiveJargbV2_1Preset::SteadyGreen,
            &extract_fixture_prefix("steady_green"),
            0x02,
            [0x00, 0xFF, 0x00],
        );
    }

    #[test]
    fn offline_live_builder_matches_steady_blue_fixture_prefix_and_store_metadata() {
        assert_live_builder_prefix(
            LiveJargbV2_1Preset::SteadyBlue,
            &extract_fixture_prefix("steady_blue"),
            0x02,
            [0x00, 0x00, 0xFF],
        );
    }

    #[test]
    fn offline_live_builder_matches_breath_red_fixture_prefix_and_store_metadata() {
        assert_live_builder_prefix(
            LiveJargbV2_1Preset::BreathRed,
            &extract_fixture_prefix("breath_red"),
            0x04,
            [0xFF, 0x00, 0x00],
        );
    }

    #[test]
    fn offline_live_builder_matches_off_retained_red_fixture_prefix_and_store_metadata() {
        assert_live_builder_prefix(
            LiveJargbV2_1Preset::OffRetainedRed,
            &extract_fixture_prefix("off_red_rgb_retained"),
            0x00,
            [0xFF, 0x00, 0x00],
        );
    }

    #[test]
    fn steady_red_green_and_blue_live_fixtures_map_to_mode_0x02_and_expected_rgb() {
        assert_eq!(
            decode_mode_preview(&extract_fixture_prefix("steady_red")),
            LiveModePreview {
                mode: 0x02,
                rgb: [0xFF, 0x00, 0x00],
            }
        );
        assert_eq!(
            decode_mode_preview(&extract_fixture_prefix("steady_green")),
            LiveModePreview {
                mode: 0x02,
                rgb: [0x00, 0xFF, 0x00],
            }
        );
        assert_eq!(
            decode_mode_preview(&extract_fixture_prefix("steady_blue")),
            LiveModePreview {
                mode: 0x02,
                rgb: [0x00, 0x00, 0xFF],
            }
        );
    }

    #[test]
    fn breath_red_live_fixture_maps_to_mode_0x04_and_ff0000() {
        assert_eq!(
            decode_mode_preview(&extract_fixture_prefix("breath_red")),
            LiveModePreview {
                mode: 0x04,
                rgb: [0xFF, 0x00, 0x00],
            }
        );
    }

    #[test]
    fn off_live_fixture_maps_to_mode_0x00_without_requiring_black_rgb() {
        let preview = decode_mode_preview(&extract_fixture_prefix("off_red_rgb_retained"));

        assert_eq!(preview.mode, 0x00);
        assert_eq!(preview.rgb, [0xFF, 0x00, 0x00]);
        assert_ne!(preview.rgb, [0x00, 0x00, 0x00]);
    }

    #[test]
    fn pcap_derived_fixture_stream_contains_no_static_gen2_or_advanced_shapes() {
        let mut bytes = parse_hex_fixture(FIXTURE_STREAM_WITH_LIVE_SETUPS).unwrap();
        for fixture in LIVE_MODE_FIXTURES {
            bytes.extend(parse_usbpcap_hexdump_fixture(fixture.bytes).unwrap());
        }
        let absent_shapes = [
            ReportShape {
                report_id: 0x90,
                report_length: 302,
            },
            ReportShape {
                report_id: 0x91,
                report_length: 302,
            },
            ReportShape {
                report_id: 0x92,
                report_length: 302,
            },
            ReportShape {
                report_id: 0x93,
                report_length: 302,
            },
            ReportShape {
                report_id: 0x51,
                report_length: 727,
            },
            ReportShape {
                report_id: 0xB0,
                report_length: 761,
            },
        ];

        assert!(contains_report_shape(
            &bytes,
            ReportShape {
                report_id: 0x50,
                report_length: 290
            }
        ));

        for shape in absent_shapes {
            assert!(
                !contains_report_shape(&bytes, shape),
                "fixture unexpectedly contains report 0x{:02X}/{}",
                shape.report_id,
                shape.report_length
            );
        }
    }

    #[test]
    fn frames_4781_and_7757_are_live_confirmed_0x50_reports_with_different_store_bytes() {
        let setup = super::find_usb_setup_packets(&parse_hex_fixture(SETUP_0X50_290).unwrap())[0];
        let frame_4781 = CapturedReportSummary {
            frame: 4781,
            setup,
            payload_prefix: parse_hex_fixture(FRAME_4781_PREFIX).unwrap(),
            report_length: GEN1_REPORT_LENGTH,
            store_byte: 0x00,
        };
        let frame_7757 = CapturedReportSummary {
            frame: 7757,
            setup,
            payload_prefix: parse_hex_fixture(FRAME_7757_PREFIX).unwrap(),
            report_length: GEN1_REPORT_LENGTH,
            store_byte: 0x01,
        };

        assert!(frame_4781.is_live_confirmed_0x50_report());
        assert!(frame_7757.is_live_confirmed_0x50_report());
        assert_eq!(frame_4781.store_byte, 0x00);
        assert_eq!(frame_7757.store_byte, 0x01);
        assert_ne!(frame_4781.store_byte, frame_7757.store_byte);
    }

    #[test]
    fn jargb_v2_1_to_0x90_is_not_live_confirmed_by_capture_fixtures() {
        let mut bytes = parse_hex_fixture(FIXTURE_STREAM_WITH_LIVE_SETUPS).unwrap();
        for fixture in LIVE_MODE_FIXTURES {
            bytes.extend(parse_usbpcap_hexdump_fixture(fixture.bytes).unwrap());
        }

        assert!(contains_report_shape(
            &bytes,
            ReportShape {
                report_id: 0x50,
                report_length: 290
            }
        ));
        assert!(!contains_report_shape(
            &bytes,
            ReportShape {
                report_id: 0x90,
                report_length: 302
            }
        ));
    }

    fn decode_mode_preview(prefix: &str) -> LiveModePreview {
        let bytes = parse_hex_fixture(prefix).unwrap();

        LiveModePreview {
            mode: bytes[1],
            rgb: [bytes[2], bytes[3], bytes[4]],
        }
    }

    fn assert_live_builder_prefix(
        preset: LiveJargbV2_1Preset,
        fixture_prefix: &str,
        expected_mode: u8,
        expected_rgb: [u8; 3],
    ) {
        let payload = build_live_confirmed_jargb_v2_1_0x50_payload(preset);
        let fixture = parse_hex_fixture(fixture_prefix).unwrap();

        assert_eq!(payload.len(), GEN1_REPORT_LENGTH);
        assert_eq!(payload[0], GEN1_REPORT_ID);
        assert_eq!(payload[1], expected_mode);
        assert_eq!(&payload[2..5], &expected_rgb);
        assert_eq!(&payload[..fixture.len()], fixture.as_slice());
        assert_eq!(payload[GEN1_REPORT_LENGTH - 1], 0x01);
    }

    fn extract_fixture_prefix(label: &str) -> String {
        let fixture = LIVE_MODE_FIXTURES
            .iter()
            .find(|fixture| fixture.label == label)
            .unwrap_or_else(|| panic!("missing live fixture {label}"));
        let extracted =
            extract_0x50_payload(&parse_usbpcap_hexdump_fixture(fixture.bytes).unwrap()).unwrap();

        extracted.payload[..5]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn first_difference_offset_reports_first_byte_difference() {
        assert_eq!(
            super::first_difference_offset(&[0x50, 0x02], &[0x50, 0x04]),
            Some(1)
        );
    }

    #[test]
    fn first_difference_offset_reports_length_difference_after_shared_prefix() {
        assert_eq!(
            super::first_difference_offset(&[0x50, 0x02], &[0x50, 0x02, 0xFF]),
            Some(2)
        );
    }

    #[test]
    fn first_difference_offset_reports_none_for_equal_buffers() {
        assert_eq!(
            super::first_difference_offset(&[0x50, 0x02], &[0x50, 0x02]),
            None
        );
    }

    fn differing_offsets(comparisons: &[super::ByteComparison]) -> Vec<usize> {
        comparisons
            .iter()
            .filter(|comparison| comparison.status == ByteComparisonStatus::Differs)
            .map(|comparison| comparison.offset)
            .collect()
    }
}
