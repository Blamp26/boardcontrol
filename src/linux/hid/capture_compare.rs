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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedHidPayload {
    pub setup: Option<UsbSetupPacket>,
    pub setup_offset: Option<usize>,
    pub payload_offset: usize,
    pub payload: Vec<u8>,
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

#[cfg(test)]
mod tests {
    use crate::linux::hid::report::{
        GEN1_REPORT_LENGTH, Ms7e75Zone, RgbColor, build_zone_static_rgb_report,
    };

    use super::{
        ByteComparisonStatus, compare_0x50_payload_to_gen1_builder, extract_0x50_payload,
        parse_hex_fixture,
    };

    const FRAME_4781_PREFIX: &str = "50 02 14 ff 09 00 ff";
    const FRAME_7757_PREFIX: &str = "50 03 ff 00 00 ff 64";
    const FRAME_4781_WITH_SETUP: &str = "21 09 50 03 00 00 22 01 50 02 14 ff 09 00 ff";

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
        assert_eq!(differing_offsets, vec![1, 2, 3, 4, 6]);
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
        assert_eq!(differing_offsets, vec![1, 2, 5, 6]);
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

    fn differing_offsets(comparisons: &[super::ByteComparison]) -> Vec<usize> {
        comparisons
            .iter()
            .filter(|comparison| comparison.status == ByteComparisonStatus::Differs)
            .map(|comparison| comparison.offset)
            .collect()
    }
}
