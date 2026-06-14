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

pub fn read_dmi_info() -> Result<DmiInfo> {
    Ok(DmiInfo {
        board_vendor: read_optional_dmi_field("/sys/class/dmi/id/board_vendor"),
        board_name: read_optional_dmi_field("/sys/class/dmi/id/board_name"),
        board_version: read_optional_dmi_field("/sys/class/dmi/id/board_version"),
        product_name: read_optional_dmi_field("/sys/class/dmi/id/product_name"),
    })
}

fn read_optional_dmi_field(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::DmiInfo;

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
}
