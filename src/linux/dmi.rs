use std::fs;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmiInfo {
    pub board_vendor: Option<String>,
    pub board_name: Option<String>,
    pub board_version: Option<String>,
    pub product_name: Option<String>,
}

impl DmiInfo {
    pub fn looks_like_msi_7a45(&self) -> bool {
        let vendor_matches = self
            .board_vendor
            .as_deref()
            .map(|value| {
                let lower = value.to_ascii_lowercase();
                lower.contains("msi") || lower.contains("micro-star")
            })
            .unwrap_or(false);

        let board_matches = [
            self.board_name.as_deref(),
            self.board_version.as_deref(),
            self.product_name.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|value| value.to_ascii_lowercase().contains("7a45"));

        vendor_matches && board_matches
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightStatus {
    Pass,
    Blocked(String),
}

pub fn read_dmi_info() -> Result<DmiInfo> {
    Ok(DmiInfo {
        board_vendor: read_optional_dmi_field("/sys/class/dmi/id/board_vendor"),
        board_name: read_optional_dmi_field("/sys/class/dmi/id/board_name"),
        board_version: read_optional_dmi_field("/sys/class/dmi/id/board_version"),
        product_name: read_optional_dmi_field("/sys/class/dmi/id/product_name"),
    })
}

pub fn evaluate_hardware_read_preflight(
    dmi: &DmiInfo,
    superio_ports_available: bool,
) -> PreflightStatus {
    if !dmi.looks_like_msi_7a45() {
        return PreflightStatus::Blocked(format!(
            "host DMI does not look like MSI 7A45: vendor={} board={} product={}",
            dmi.board_vendor.as_deref().unwrap_or("unknown"),
            dmi.board_name
                .as_deref()
                .or(dmi.board_version.as_deref())
                .unwrap_or("unknown"),
            dmi.product_name.as_deref().unwrap_or("unknown"),
        ));
    }

    if !superio_ports_available {
        return PreflightStatus::Blocked("Super I/O ports 004e-004f are busy".to_string());
    }

    PreflightStatus::Pass
}

fn read_optional_dmi_field(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{DmiInfo, PreflightStatus, evaluate_hardware_read_preflight};

    #[test]
    fn msi_positive() {
        let dmi = DmiInfo {
            board_vendor: Some("Micro-Star International Co., Ltd.".to_string()),
            board_name: Some("MS-7A45".to_string()),
            board_version: None,
            product_name: None,
        };
        assert!(dmi.looks_like_msi_7a45());
    }

    #[test]
    fn msi_lowercase_positive() {
        let dmi = DmiInfo {
            board_vendor: Some("msi".to_string()),
            board_name: Some("7a45".to_string()),
            board_version: None,
            product_name: None,
        };
        assert!(dmi.looks_like_msi_7a45());
    }

    #[test]
    fn dell_negative() {
        let dmi = DmiInfo {
            board_vendor: Some("Dell Inc.".to_string()),
            board_name: Some("03V7GF".to_string()),
            board_version: None,
            product_name: Some("OptiPlex 5000".to_string()),
        };
        assert!(!dmi.looks_like_msi_7a45());
    }

    #[test]
    fn msi_but_no_7a45_negative() {
        let dmi = DmiInfo {
            board_vendor: Some("MSI".to_string()),
            board_name: Some("7B79".to_string()),
            board_version: None,
            product_name: None,
        };
        assert!(!dmi.looks_like_msi_7a45());
    }

    #[test]
    fn non_msi_vendor_negative() {
        let dmi = DmiInfo {
            board_vendor: Some("Dell".to_string()),
            board_name: Some("7A45".to_string()),
            board_version: None,
            product_name: None,
        };
        assert!(!dmi.looks_like_msi_7a45());
    }

    #[test]
    fn preflight_dell_blocked_by_dmi() {
        let dmi = DmiInfo {
            board_vendor: Some("Dell Inc.".to_string()),
            board_name: Some("03V7GF".to_string()),
            board_version: None,
            product_name: Some("OptiPlex 5000".to_string()),
        };
        match evaluate_hardware_read_preflight(&dmi, true) {
            PreflightStatus::Blocked(reason) => {
                assert!(reason.contains("host DMI does not look like MSI 7A45"));
            }
            PreflightStatus::Pass => panic!("expected blocked"),
        }
    }

    #[test]
    fn preflight_msi_7a45_and_ports_available_pass() {
        let dmi = DmiInfo {
            board_vendor: Some("Micro-Star International Co., Ltd.".to_string()),
            board_name: Some("MS-7A45".to_string()),
            board_version: None,
            product_name: None,
        };
        assert!(matches!(
            evaluate_hardware_read_preflight(&dmi, true),
            PreflightStatus::Pass
        ));
    }

    #[test]
    fn preflight_msi_7a45_but_ports_busy_blocked() {
        let dmi = DmiInfo {
            board_vendor: Some("MSI".to_string()),
            board_name: Some("7A45".to_string()),
            board_version: None,
            product_name: None,
        };
        match evaluate_hardware_read_preflight(&dmi, false) {
            PreflightStatus::Blocked(reason) => {
                assert_eq!(reason, "Super I/O ports 004e-004f are busy");
            }
            PreflightStatus::Pass => panic!("expected blocked"),
        }
    }

    #[test]
    fn preflight_dmi_reason_first() {
        let dmi = DmiInfo {
            board_vendor: Some("Dell".to_string()),
            board_name: Some("7A45".to_string()),
            board_version: None,
            product_name: Some("OptiPlex 5000".to_string()),
        };
        match evaluate_hardware_read_preflight(&dmi, false) {
            PreflightStatus::Blocked(reason) => {
                assert!(reason.contains("host DMI does not look like MSI 7A45"));
            }
            PreflightStatus::Pass => panic!("expected blocked"),
        }
    }
}
