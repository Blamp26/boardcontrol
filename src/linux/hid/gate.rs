use crate::error::Result;
use crate::linux::dmi::DmiInfo;
use crate::linux::hid::inventory::{
    HidInventoryCandidate, SerialPrefixStatus, TARGET_BOARD_ID, TARGET_COLLECTION_NUMBER,
    TARGET_INTERFACE_NUMBER, TARGET_PRODUCT_ID, TARGET_VENDOR_ID, inventory_candidates,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidGateStatus {
    EligibleForDryRun,
    Blocked,
    Inconclusive,
}

impl HidGateStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EligibleForDryRun => "eligible_for_dry_run",
            Self::Blocked => "blocked",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmiGateResult {
    pub matched: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidInventoryGateResult {
    pub matched: bool,
    pub summary: String,
    pub candidate_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerialGateResult {
    pub status: HidGateStatus,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HidBoardGateResult {
    pub status: HidGateStatus,
    pub dmi: DmiGateResult,
    pub inventory: HidInventoryGateResult,
    pub serial: SerialGateResult,
}

pub fn read_hid_board_gate() -> Result<HidBoardGateResult> {
    let dmi = crate::linux::dmi::read_dmi_info()?;
    let candidates = inventory_candidates()?;
    Ok(evaluate_hid_board_gate(&dmi, &candidates))
}

pub fn evaluate_hid_board_gate(
    dmi: &DmiInfo,
    candidates: &[HidInventoryCandidate],
) -> HidBoardGateResult {
    let dmi_result = evaluate_dmi_gate(dmi);
    if !dmi_result.matched {
        return HidBoardGateResult {
            status: HidGateStatus::Blocked,
            dmi: dmi_result,
            inventory: HidInventoryGateResult {
                matched: false,
                summary: "skipped because DMI gate is blocked".to_string(),
                candidate_count: candidates.len(),
            },
            serial: SerialGateResult {
                status: HidGateStatus::Blocked,
                summary: "skipped because DMI gate is blocked".to_string(),
            },
        };
    }

    let matching_candidates = matching_gate_candidates(candidates);
    let inventory_result = evaluate_inventory_gate(candidates, &matching_candidates);
    if !inventory_result.matched {
        return HidBoardGateResult {
            status: HidGateStatus::Blocked,
            dmi: dmi_result,
            inventory: inventory_result,
            serial: SerialGateResult {
                status: HidGateStatus::Blocked,
                summary: "blocked because no candidate passed the HID identity gate".to_string(),
            },
        };
    }

    let serial_result = evaluate_serial_gate(&matching_candidates);
    let status = serial_result.status;

    HidBoardGateResult {
        status,
        dmi: dmi_result,
        inventory: inventory_result,
        serial: serial_result,
    }
}

pub fn format_gate_report(result: &HidBoardGateResult) -> String {
    let next_safe_command = if result.status == HidGateStatus::EligibleForDryRun {
        "msi-ml linux hid dry-run --zone JRGB1 --color ff0000"
    } else {
        "none"
    };

    [
        "MS-7E75 HID board gate".to_string(),
        "  status = READ ONLY".to_string(),
        "  mode = read-only metadata and DMI checks only".to_string(),
        format!("  dmi_match = {}", result.dmi.matched),
        format!("  dmi = {}", result.dmi.summary),
        format!("  hid_inventory_match = {}", result.inventory.matched),
        format!("  hid_inventory = {}", result.inventory.summary),
        format!("  hid_candidates = {}", result.inventory.candidate_count),
        format!("  serial_gate = {}", result.serial.summary),
        format!("  final_status = {}", result.status.as_str()),
        "  devices_opened = no".to_string(),
        "  writes_enabled = no".to_string(),
        "  writes_performed = no".to_string(),
        "  support = unsupported/not enabled".to_string(),
        "  message = Phase 2 gate is read-only; HID writes remain disabled".to_string(),
        format!("  next_safe_command = {next_safe_command}"),
    ]
    .join("\n")
}

fn evaluate_dmi_gate(dmi: &DmiInfo) -> DmiGateResult {
    let vendor = dmi.board_vendor.as_deref().unwrap_or("unknown");
    let board_name = dmi.board_name.as_deref().unwrap_or("unknown");
    let product_name = dmi.product_name.as_deref().unwrap_or("unknown");

    let vendor_matches = dmi
        .board_vendor
        .as_deref()
        .map(|value| {
            let lower = value.to_ascii_lowercase();
            lower.contains("msi") || lower.contains("micro-star")
        })
        .unwrap_or(false);
    let board_name_matches = dmi
        .board_name
        .as_deref()
        .map(|value| {
            value
                .to_ascii_lowercase()
                .contains("b850 gaming plus wifi pz")
        })
        .unwrap_or(false);
    let product_matches = dmi
        .product_name
        .as_deref()
        .map(|value| {
            let lower = value.to_ascii_lowercase();
            lower.contains("ms-7e75") || lower.contains("7e75")
        })
        .unwrap_or(false);

    let matched = vendor_matches && board_name_matches && product_matches;
    let summary = if matched {
        format!(
            "matched MSI MS-7E75 board identity: vendor={vendor} board={board_name} product={product_name}"
        )
    } else {
        format!(
            "blocked: expected MSI B850 GAMING PLUS WIFI PZ / MS-7E75, got vendor={vendor} board={board_name} product={product_name}"
        )
    };

    DmiGateResult { matched, summary }
}

fn matching_gate_candidates(candidates: &[HidInventoryCandidate]) -> Vec<&HidInventoryCandidate> {
    candidates
        .iter()
        .filter(|candidate| {
            candidate.vendor_id == TARGET_VENDOR_ID
                && candidate.product_id == TARGET_PRODUCT_ID
                && candidate
                    .interface_number
                    .is_none_or(|interface_number| interface_number == TARGET_INTERFACE_NUMBER)
                && candidate
                    .collection_number
                    .is_none_or(|collection_number| collection_number == TARGET_COLLECTION_NUMBER)
        })
        .collect()
}

fn evaluate_inventory_gate(
    all_candidates: &[HidInventoryCandidate],
    matching_candidates: &[&HidInventoryCandidate],
) -> HidInventoryGateResult {
    if matching_candidates.is_empty() {
        return HidInventoryGateResult {
            matched: false,
            summary: format!(
                "blocked: no candidate matched VID=0x{TARGET_VENDOR_ID:04X} PID=0x{TARGET_PRODUCT_ID:04X} with MI/COL gate"
            ),
            candidate_count: all_candidates.len(),
        };
    }

    HidInventoryGateResult {
        matched: true,
        summary: format!(
            "matched {} candidate(s) with VID=0x{TARGET_VENDOR_ID:04X} PID=0x{TARGET_PRODUCT_ID:04X}",
            matching_candidates.len()
        ),
        candidate_count: matching_candidates.len(),
    }
}

fn evaluate_serial_gate(candidates: &[&HidInventoryCandidate]) -> SerialGateResult {
    if candidates.iter().any(|candidate| {
        matches!(
            candidate.serial_prefix,
            SerialPrefixStatus::ExpectedBoardId(TARGET_BOARD_ID)
        )
    }) {
        return SerialGateResult {
            status: HidGateStatus::EligibleForDryRun,
            summary: format!("matched expected board id 0x{TARGET_BOARD_ID:04X}"),
        };
    }

    if candidates.iter().any(|candidate| {
        matches!(
            candidate.serial_prefix,
            SerialPrefixStatus::UnexpectedBoardId(_) | SerialPrefixStatus::Invalid(_)
        )
    }) {
        let candidate = candidates
            .iter()
            .find(|candidate| {
                matches!(
                    candidate.serial_prefix,
                    SerialPrefixStatus::UnexpectedBoardId(_) | SerialPrefixStatus::Invalid(_)
                )
            })
            .expect("candidate with invalid serial gate should exist");
        return SerialGateResult {
            status: HidGateStatus::Blocked,
            summary: format!("blocked: {}", candidate.serial_prefix.summary()),
        };
    }

    SerialGateResult {
        status: HidGateStatus::Inconclusive,
        summary:
            "inconclusive: serial missing or unreadable; expected board id 0x7E75 was not confirmed"
                .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::linux::dmi::DmiInfo;
    use crate::linux::hid::inventory::{
        HidInventoryCandidate, SerialPrefixStatus, TARGET_BOARD_ID, TARGET_COLLECTION_NUMBER,
        TARGET_INTERFACE_NUMBER, TARGET_PRODUCT_ID, TARGET_VENDOR_ID,
    };

    use super::{HidGateStatus, evaluate_hid_board_gate, format_gate_report};

    #[test]
    fn matching_dmi_and_hid_and_serial_are_eligible_for_dry_run() {
        let result = evaluate_hid_board_gate(&matching_dmi(), &[matching_candidate()]);

        assert_eq!(result.status, HidGateStatus::EligibleForDryRun);
    }

    #[test]
    fn wrong_dmi_is_blocked() {
        let dmi = DmiInfo {
            board_vendor: Some("Dell".to_string()),
            board_name: Some("B850 GAMING PLUS WIFI PZ (MS-7E75)".to_string()),
            board_version: None,
            product_name: Some("MS-7E75".to_string()),
        };

        let result = evaluate_hid_board_gate(&dmi, &[matching_candidate()]);

        assert_eq!(result.status, HidGateStatus::Blocked);
        assert!(!result.dmi.matched);
    }

    #[test]
    fn wrong_vid_pid_is_blocked() {
        let candidate = HidInventoryCandidate {
            vendor_id: 0x1234,
            ..matching_candidate()
        };

        let result = evaluate_hid_board_gate(&matching_dmi(), &[candidate]);

        assert_eq!(result.status, HidGateStatus::Blocked);
        assert!(!result.inventory.matched);
    }

    #[test]
    fn wrong_serial_board_id_is_blocked() {
        let candidate = HidInventoryCandidate {
            serial_prefix: SerialPrefixStatus::UnexpectedBoardId(0x7E76),
            serial_number: Some("7E76ABCD".to_string()),
            ..matching_candidate()
        };

        let result = evaluate_hid_board_gate(&matching_dmi(), &[candidate]);

        assert_eq!(result.status, HidGateStatus::Blocked);
        assert_eq!(result.serial.status, HidGateStatus::Blocked);
    }

    #[test]
    fn missing_serial_is_inconclusive() {
        let candidate = HidInventoryCandidate {
            serial_prefix: SerialPrefixStatus::Missing,
            serial_number: None,
            ..matching_candidate()
        };

        let result = evaluate_hid_board_gate(&matching_dmi(), &[candidate]);

        assert_eq!(result.status, HidGateStatus::Inconclusive);
        assert_eq!(result.serial.status, HidGateStatus::Inconclusive);
    }

    #[test]
    fn mi_or_col_mismatch_is_blocked() {
        let mi_mismatch = HidInventoryCandidate {
            interface_number: Some(1),
            ..matching_candidate()
        };
        let col_mismatch = HidInventoryCandidate {
            collection_number: Some(1),
            ..matching_candidate()
        };

        let mi_result = evaluate_hid_board_gate(&matching_dmi(), &[mi_mismatch]);
        let col_result = evaluate_hid_board_gate(&matching_dmi(), &[col_mismatch]);

        assert_eq!(mi_result.status, HidGateStatus::Blocked);
        assert_eq!(col_result.status, HidGateStatus::Blocked);
    }

    #[test]
    fn report_is_explicitly_read_only_and_disabled() {
        let report = format_gate_report(&evaluate_hid_board_gate(
            &matching_dmi(),
            &[matching_candidate()],
        ));

        assert!(report.contains("status = READ ONLY"));
        assert!(report.contains("mode = read-only metadata and DMI checks only"));
        assert!(report.contains("devices_opened = no"));
        assert!(report.contains("writes_enabled = no"));
        assert!(report.contains("writes_performed = no"));
        assert!(report.contains("support = unsupported/not enabled"));
        assert!(
            report.contains(
                "next_safe_command = msi-ml linux hid dry-run --zone JRGB1 --color ff0000"
            )
        );
    }

    fn matching_dmi() -> DmiInfo {
        DmiInfo {
            board_vendor: Some("Micro-Star International Co., Ltd.".to_string()),
            board_name: Some("B850 GAMING PLUS WIFI PZ (MS-7E75)".to_string()),
            board_version: None,
            product_name: Some("MS-7E75".to_string()),
        }
    }

    fn matching_candidate() -> HidInventoryCandidate {
        HidInventoryCandidate {
            syspath: PathBuf::from("/sys/bus/hid/devices/0003:0DB0:0076.0001"),
            vendor_id: TARGET_VENDOR_ID,
            product_id: TARGET_PRODUCT_ID,
            interface_number: Some(TARGET_INTERFACE_NUMBER),
            collection_number: Some(TARGET_COLLECTION_NUMBER),
            product_name: Some("MSI Common HID".to_string()),
            serial_number: Some("7E75ABCD".to_string()),
            serial_prefix: SerialPrefixStatus::ExpectedBoardId(TARGET_BOARD_ID),
        }
    }
}
