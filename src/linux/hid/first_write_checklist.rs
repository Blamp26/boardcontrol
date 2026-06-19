pub const BOARD_PROFILE: &str = "MS-7E75";
pub const PHASE4_STATUS: &str = "HOLD";
pub const FIRST_WRITE_READY: &str = "no";

const ALREADY_SATISFIED: &[&str] = &[
    "USBPcap live MSI Center evidence confirms SET_REPORT 0x50 with HID payload length 290 for JARGB_V2_1.",
    "Full TEST 2 through TEST 6 setup+payload USBPcap fixtures are checked in for steady red/green/blue, breath red, and off with retained red.",
    "The isolated offline JARGB_V2_1 live builder matches the checked-in 290-byte payloads byte-for-byte.",
    "exact-live-dry-run prints the exact setup bytes and full 290-byte payload with fixture_match = yes.",
    "Unsupported zones, modes, and colors reject explicitly with no guessing or fallback payload.",
    "The HID safety tripwire remains present and is not bypassed by the offline builder or checklist path.",
];

const STILL_REQUIRED: &[&str] = &[
    "Before any future write test, the user must close MSI Center, OpenRGB, and SignalRGB.",
    "The user must confirm the exact board/profile is MS-7E75 before any future write test.",
    "The first write may only target JARGB_V2_1.",
    "The first write may only use steady ff0000.",
    "The first write may send one command and one packet only, with no loop and no retry.",
    "Any future write path must require a scary explicit flag before sending anything.",
    "Any future write path must refuse unsupported zones, modes, and colors.",
    "Any future write path must print the exact payload before sending it.",
    "Any future write path must print a last-chance-to-abort warning immediately before send.",
    "Any future write path must not support 0x90..0x93.",
];

const USER_DECISION_REQUIRED: &[&str] = &[
    "A separate explicit user risk decision is still required before Phase 4 can move off HOLD.",
    "Byte-for-byte offline equality and exact dry-run output do not by themselves approve hardware writes.",
    "No HID write implementation, single-use write path, HID feature-call path, or device-open path may be added without that separate approval.",
];

pub fn format_first_write_checklist() -> String {
    let mut lines = vec![
        "MS-7E75 first-write checklist".to_string(),
        format!("board_profile = {BOARD_PROFILE}"),
        "status = READ ONLY".to_string(),
        "devices_opened = no".to_string(),
        "writes_enabled = no".to_string(),
        "writes_performed = no".to_string(),
        format!("phase4_status = {PHASE4_STATUS}"),
        format!("first_write_ready = {FIRST_WRITE_READY}"),
        String::new(),
        "already_satisfied:".to_string(),
    ];

    append_section(&mut lines, ALREADY_SATISFIED);
    lines.push(String::new());
    lines.push("still_required_before_first_write:".to_string());
    append_section(&mut lines, STILL_REQUIRED);
    lines.push(String::new());
    lines.push("explicit_user_decision_required:".to_string());
    append_section(&mut lines, USER_DECISION_REQUIRED);

    lines.join("\n")
}

fn append_section(lines: &mut Vec<String>, items: &[&str]) {
    for item in items {
        lines.push(format!("- {item}"));
    }
}

#[cfg(test)]
mod tests {
    use super::format_first_write_checklist;

    #[test]
    fn checklist_output_contains_required_safety_status() {
        let output = format_first_write_checklist();

        assert!(output.contains("status = READ ONLY"));
        assert!(output.contains("devices_opened = no"));
        assert!(output.contains("writes_enabled = no"));
        assert!(output.contains("writes_performed = no"));
        assert!(output.contains("phase4_status = HOLD"));
        assert!(output.contains("first_write_ready = no"));
    }

    #[test]
    fn checklist_output_contains_required_sections() {
        let output = format_first_write_checklist();

        assert!(output.contains("already_satisfied:"));
        assert!(output.contains("still_required_before_first_write:"));
        assert!(output.contains("explicit_user_decision_required:"));
        assert!(output.contains("0x50 with HID payload length 290"));
        assert!(output.contains("one command and one packet only"));
        assert!(output.contains("separate explicit user risk decision"));
    }
}
