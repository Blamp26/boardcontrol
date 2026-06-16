#![allow(dead_code)]

pub const GEN1_REPORT_ID: u8 = 0x50;
pub const GEN1_REPORT_LENGTH: usize = 290;
pub const GEN1_AREA_RECORD_SIZE: usize = 16;
pub const JRGB1_AREA_INDEX: u8 = 9;

pub const GEN2_REPORT_BASE_ID: u8 = 0x90;
pub const GEN2_REPORT_LENGTH: usize = 302;
pub const GEN2_STRIP_RECORD_SIZE: usize = 20;
pub const GEN2_MAX_STRIPS: usize = 6;
pub const GEN2_MAX_LED_COUNT: u8 = 180;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportBuildError {
    UnsupportedZone(String),
    ZoneTypeMismatch {
        zone: Ms7e75Zone,
        expected: &'static str,
    },
    InvalidColorCount(u8),
    InvalidLedCount(u8),
    TooManyGen2Strips(usize),
}

impl std::fmt::Display for ReportBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedZone(zone) => write!(f, "unsupported MS-7E75 zone: {zone}"),
            Self::ZoneTypeMismatch { zone, expected } => {
                write!(f, "zone {zone} does not support {expected} reports")
            }
            Self::InvalidColorCount(count) => {
                write!(f, "color count must be between 1 and 4, got {count}")
            }
            Self::InvalidLedCount(count) => {
                write!(
                    f,
                    "LED count must be between 1 and {GEN2_MAX_LED_COUNT}, got {count}"
                )
            }
            Self::TooManyGen2Strips(count) => {
                write!(
                    f,
                    "Gen2 reports support at most {GEN2_MAX_STRIPS} strips, got {count}"
                )
            }
        }
    }
}

impl std::error::Error for ReportBuildError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ms7e75Zone {
    Jrgb1,
    JargbV2_1,
    JargbV2_2,
    JargbV2_3,
    EzConn,
}

impl Ms7e75Zone {
    pub fn from_name(name: &str) -> Result<Self, ReportBuildError> {
        match name {
            "JRGB1" => Ok(Self::Jrgb1),
            "JARGB_V2_1" => Ok(Self::JargbV2_1),
            "JARGB_V2_2" => Ok(Self::JargbV2_2),
            "JARGB_V2_3" => Ok(Self::JargbV2_3),
            "EZ Conn" => Ok(Self::EzConn),
            _ => Err(ReportBuildError::UnsupportedZone(name.to_string())),
        }
    }

    pub const fn report_id(self) -> u8 {
        match self {
            Self::Jrgb1 => GEN1_REPORT_ID,
            Self::JargbV2_1 => GEN2_REPORT_BASE_ID,
            Self::JargbV2_2 => GEN2_REPORT_BASE_ID + 1,
            Self::JargbV2_3 => GEN2_REPORT_BASE_ID + 2,
            Self::EzConn => GEN2_REPORT_BASE_ID + 3,
        }
    }

    pub const fn gen1_area_index(self) -> Option<u8> {
        match self {
            Self::Jrgb1 => Some(JRGB1_AREA_INDEX),
            Self::JargbV2_1 | Self::JargbV2_2 | Self::JargbV2_3 | Self::EzConn => None,
        }
    }

    pub const fn gen2_port_index(self) -> Option<u8> {
        match self {
            Self::Jrgb1 => None,
            Self::JargbV2_1 => Some(0),
            Self::JargbV2_2 => Some(1),
            Self::JargbV2_3 => Some(2),
            Self::EzConn => Some(3),
        }
    }
}

impl std::fmt::Display for Ms7e75Zone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Jrgb1 => "JRGB1",
            Self::JargbV2_1 => "JARGB_V2_1",
            Self::JargbV2_2 => "JARGB_V2_2",
            Self::JargbV2_3 => "JARGB_V2_3",
            Self::EzConn => "EZ Conn",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LightingMode {
    Off = 0,
    Wave = 1,
    Steady = 2,
    Flame = 3,
    Breathing = 4,
    ColorRing = 5,
    Lightning = 6,
    Recreation = 7,
    Meteor = 8,
    Advanced = 9,
    GodLike = 10,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gen1AreaSetting {
    pub mode: LightingMode,
    pub colors: [RgbColor; 4],
    pub color_count: u8,
    pub option_2: u8,
    pub cycle_number: u8,
}

impl Gen1AreaSetting {
    pub const fn static_rgb(color: RgbColor) -> Self {
        Self {
            mode: LightingMode::Steady,
            colors: [
                color,
                RgbColor::new(0, 0, 0),
                RgbColor::new(0, 0, 0),
                RgbColor::new(0, 0, 0),
            ],
            color_count: 1,
            option_2: 0,
            cycle_number: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gen2StripSetting {
    pub fixed_id: u32,
    pub mode: LightingMode,
    pub colors: [RgbColor; 4],
    pub color_count: u8,
    pub option_2: u8,
    pub led_count: u8,
}

impl Gen2StripSetting {
    pub const fn static_rgb(color: RgbColor, led_count: u8) -> Self {
        Self {
            fixed_id: 0,
            mode: LightingMode::Steady,
            colors: [
                color,
                RgbColor::new(0, 0, 0),
                RgbColor::new(0, 0, 0),
                RgbColor::new(0, 0, 0),
            ],
            color_count: 1,
            option_2: 0,
            led_count,
        }
    }
}

pub fn build_zone_static_rgb_report(
    zone: Ms7e75Zone,
    color: RgbColor,
    store: bool,
) -> Result<Vec<u8>, ReportBuildError> {
    if zone == Ms7e75Zone::Jrgb1 {
        Ok(build_gen1_report(zone, &Gen1AreaSetting::static_rgb(color), store)?.to_vec())
    } else {
        Ok(build_gen2_report(zone, &[Gen2StripSetting::static_rgb(color, 1)], store)?.to_vec())
    }
}

pub fn build_gen1_report(
    zone: Ms7e75Zone,
    area: &Gen1AreaSetting,
    store: bool,
) -> Result<[u8; GEN1_REPORT_LENGTH], ReportBuildError> {
    let area_index = zone
        .gen1_area_index()
        .ok_or(ReportBuildError::ZoneTypeMismatch {
            zone,
            expected: "Gen1",
        })?;
    validate_color_count(area.color_count)?;

    let mut buffer = [0_u8; GEN1_REPORT_LENGTH];
    buffer[0] = zone.report_id();

    let base = usize::from(area_index) * GEN1_AREA_RECORD_SIZE;
    buffer[base + 1] = area.mode as u8;
    write_colors(&mut buffer, base + 2, &area.colors);
    buffer[base + 14] = area.color_count - 1;
    buffer[base + 15] = area.option_2;
    buffer[base + 16] = area.cycle_number;
    buffer[GEN1_REPORT_LENGTH - 1] = u8::from(store);

    Ok(buffer)
}

pub fn build_gen2_report(
    zone: Ms7e75Zone,
    strips: &[Gen2StripSetting],
    store: bool,
) -> Result<[u8; GEN2_REPORT_LENGTH], ReportBuildError> {
    let port_index = zone
        .gen2_port_index()
        .ok_or(ReportBuildError::ZoneTypeMismatch {
            zone,
            expected: "Gen2",
        })?;

    if strips.len() > GEN2_MAX_STRIPS {
        return Err(ReportBuildError::TooManyGen2Strips(strips.len()));
    }

    let mut buffer = [0xFF_u8; GEN2_REPORT_LENGTH];
    buffer[0] = GEN2_REPORT_BASE_ID + port_index;
    buffer[GEN2_REPORT_LENGTH - 1] = u8::from(store);

    for (index, strip) in strips.iter().enumerate() {
        validate_color_count(strip.color_count)?;
        validate_led_count(strip.led_count)?;

        let base = index * GEN2_STRIP_RECORD_SIZE;
        buffer[base + 1..base + 5].copy_from_slice(&strip.fixed_id.to_le_bytes());
        buffer[base + 5] = strip.mode as u8;
        write_colors(&mut buffer, base + 6, &strip.colors);
        buffer[base + 18] = strip.color_count - 1;
        buffer[base + 19] = strip.option_2;
        buffer[base + 20] = strip.led_count;
    }

    Ok(buffer)
}

fn validate_color_count(color_count: u8) -> Result<(), ReportBuildError> {
    if (1..=4).contains(&color_count) {
        Ok(())
    } else {
        Err(ReportBuildError::InvalidColorCount(color_count))
    }
}

fn validate_led_count(led_count: u8) -> Result<(), ReportBuildError> {
    if (1..=GEN2_MAX_LED_COUNT).contains(&led_count) {
        Ok(())
    } else {
        Err(ReportBuildError::InvalidLedCount(led_count))
    }
}

fn write_colors(buffer: &mut [u8], start: usize, colors: &[RgbColor; 4]) {
    for (index, color) in colors.iter().enumerate() {
        let offset = start + index * 3;
        buffer[offset] = color.red;
        buffer[offset + 1] = color.green;
        buffer[offset + 2] = color.blue;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GEN1_REPORT_ID, GEN1_REPORT_LENGTH, GEN2_REPORT_LENGTH, JRGB1_AREA_INDEX, LightingMode,
        Ms7e75Zone, ReportBuildError, RgbColor, build_gen1_report, build_gen2_report,
        build_zone_static_rgb_report,
    };

    #[test]
    fn report_lengths_match_static_docs() {
        let gen1 =
            build_zone_static_rgb_report(Ms7e75Zone::Jrgb1, RgbColor::new(1, 2, 3), false).unwrap();
        let gen2 =
            build_zone_static_rgb_report(Ms7e75Zone::JargbV2_1, RgbColor::new(4, 5, 6), false)
                .unwrap();

        assert_eq!(gen1.len(), GEN1_REPORT_LENGTH);
        assert_eq!(gen2.len(), GEN2_REPORT_LENGTH);
    }

    #[test]
    fn report_ids_match_documented_zone_mapping() {
        let cases = [
            (Ms7e75Zone::Jrgb1, 0x50),
            (Ms7e75Zone::JargbV2_1, 0x90),
            (Ms7e75Zone::JargbV2_2, 0x91),
            (Ms7e75Zone::JargbV2_3, 0x92),
            (Ms7e75Zone::EzConn, 0x93),
        ];

        for (zone, expected_report_id) in cases {
            let buffer =
                build_zone_static_rgb_report(zone, RgbColor::new(0x11, 0x22, 0x33), true).unwrap();
            assert_eq!(buffer[0], expected_report_id);
        }
    }

    #[test]
    fn jrgb1_area_index_9_is_written_at_documented_offset() {
        let buffer =
            build_zone_static_rgb_report(Ms7e75Zone::Jrgb1, RgbColor::new(0x12, 0x34, 0x56), true)
                .unwrap();
        let base = usize::from(JRGB1_AREA_INDEX) * 16;

        assert_eq!(buffer[0], GEN1_REPORT_ID);
        assert_eq!(buffer[base + 1], LightingMode::Steady as u8);
        assert_eq!(&buffer[base + 2..base + 5], &[0x12, 0x34, 0x56]);
        assert_eq!(buffer[base + 14], 0);
        assert_eq!(buffer[base + 15], 0);
        assert_eq!(buffer[base + 16], 0);
        assert_eq!(buffer[GEN1_REPORT_LENGTH - 1], 1);
    }

    #[test]
    fn jargb_v2_port_to_report_mapping_matches_static_docs() {
        let cases = [
            (Ms7e75Zone::JargbV2_1, 0x90),
            (Ms7e75Zone::JargbV2_2, 0x91),
            (Ms7e75Zone::JargbV2_3, 0x92),
            (Ms7e75Zone::EzConn, 0x93),
        ];

        for (zone, report_id) in cases {
            let buffer =
                build_zone_static_rgb_report(zone, RgbColor::new(0xAA, 0xBB, 0xCC), false).unwrap();
            assert_eq!(buffer[0], report_id);
        }
    }

    #[test]
    fn static_rgb_gen1_buffer_sets_single_color_bytes() {
        let buffer =
            build_zone_static_rgb_report(Ms7e75Zone::Jrgb1, RgbColor::new(0x20, 0x40, 0x60), false)
                .unwrap();
        let base = usize::from(JRGB1_AREA_INDEX) * 16;

        assert_eq!(buffer[base + 1], LightingMode::Steady as u8);
        assert_eq!(
            &buffer[base + 2..base + 14],
            &[0x20, 0x40, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(buffer[base + 14], 0);
        assert_eq!(buffer[GEN1_REPORT_LENGTH - 1], 0);
    }

    #[test]
    fn static_rgb_gen2_buffer_sets_first_strip_bytes() {
        let buffer = build_zone_static_rgb_report(
            Ms7e75Zone::JargbV2_1,
            RgbColor::new(0x01, 0x02, 0x03),
            true,
        )
        .unwrap();

        assert_eq!(buffer[0], 0x90);
        assert_eq!(&buffer[1..5], &[0, 0, 0, 0]);
        assert_eq!(buffer[5], LightingMode::Steady as u8);
        assert_eq!(
            &buffer[6..18],
            &[0x01, 0x02, 0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(buffer[18], 0);
        assert_eq!(buffer[19], 0);
        assert_eq!(buffer[20], 1);
        assert_eq!(buffer[GEN2_REPORT_LENGTH - 1], 1);
        assert_eq!(buffer[21], 0xFF);
    }

    #[test]
    fn gen2_static_red_layout_matches_ledkeeper2_offsets() {
        assert_gen2_static_rgb_layout(Ms7e75Zone::JargbV2_1, 0x90, RgbColor::new(0xFF, 0x00, 0x00));
    }

    #[test]
    fn gen2_static_green_layout_matches_ledkeeper2_offsets() {
        assert_gen2_static_rgb_layout(Ms7e75Zone::JargbV2_2, 0x91, RgbColor::new(0x00, 0xFF, 0x00));
    }

    #[test]
    fn gen2_static_blue_layout_matches_ledkeeper2_offsets() {
        assert_gen2_static_rgb_layout(Ms7e75Zone::JargbV2_3, 0x92, RgbColor::new(0x00, 0x00, 0xFF));
    }

    #[test]
    fn gen2_ez_conn_report_id_and_layout_match_port_3() {
        assert_gen2_static_rgb_layout(Ms7e75Zone::EzConn, 0x93, RgbColor::new(0x10, 0x20, 0x30));
    }

    #[test]
    fn invalid_zone_names_are_rejected() {
        assert!(Ms7e75Zone::from_name("SELECT ALL").is_err());
        assert!(Ms7e75Zone::from_name("JARGB_V2_4").is_err());
    }

    #[test]
    fn gen1_builder_rejects_gen2_zone() {
        let result = build_gen1_report(
            Ms7e75Zone::JargbV2_1,
            &super::Gen1AreaSetting::static_rgb(RgbColor::new(1, 2, 3)),
            false,
        );

        assert!(matches!(
            result,
            Err(ReportBuildError::ZoneTypeMismatch {
                zone: Ms7e75Zone::JargbV2_1,
                expected: "Gen1"
            })
        ));
    }

    #[test]
    fn gen2_builder_rejects_gen1_zone() {
        let result = build_gen2_report(
            Ms7e75Zone::Jrgb1,
            &[super::Gen2StripSetting::static_rgb(
                RgbColor::new(1, 2, 3),
                1,
            )],
            false,
        );

        assert!(matches!(
            result,
            Err(ReportBuildError::ZoneTypeMismatch {
                zone: Ms7e75Zone::Jrgb1,
                expected: "Gen2"
            })
        ));
    }

    fn assert_gen2_static_rgb_layout(zone: Ms7e75Zone, report_id: u8, color: RgbColor) {
        let buffer = build_zone_static_rgb_report(zone, color, false).unwrap();

        assert_eq!(buffer.len(), GEN2_REPORT_LENGTH);
        assert_eq!(buffer[0], report_id);

        // Static LEDKeeper2 evidence: Gen2_ApplyPort writes a 20-byte strip
        // record at j * 20 + 1 and leaves unused strip bytes at 0xFF.
        assert_eq!(&buffer[1..5], &[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(buffer[5], LightingMode::Steady as u8);
        assert_eq!(&buffer[6..9], &[color.red, color.green, color.blue]);
        assert_eq!(&buffer[9..18], &[0x00; 9]);
        assert_eq!(buffer[18], 0x00);
        assert_eq!(buffer[19], 0x00);
        assert_eq!(buffer[20], 0x01);
        assert_eq!(buffer[21], 0xFF);
        assert_eq!(buffer[120], 0xFF);

        // Static LEDKeeper2 evidence: Store is byte 301; Linux semantics for
        // future dispatch remain unknown because this builder is in-memory only.
        assert_eq!(buffer[GEN2_REPORT_LENGTH - 1], 0x00);
    }
}
