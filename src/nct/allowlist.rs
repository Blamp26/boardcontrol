pub fn allowed_change_mask(ldn: u8, reg: u8) -> Option<u8> {
    match (ldn, reg) {
        (0x09, 0xE0) => Some(0x80),
        (0x09, 0xE9) => Some(0x80),
        (0x09, 0x27) => Some(0x10),
        (0x09, 0x1B) => Some(0x40),
        (0x09, 0x30) => Some(0x02),
        (0x09, 0x2A) => Some(0x40),
        (0x08, 0xF0) => Some(0x80),
        (0x08, 0xF1) => Some(0x80),
        (0x0B, 0xF7) => Some(0x80),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::allowed_change_mask;

    #[test]
    fn allowlist_matches_expected_entries() {
        assert_eq!(allowed_change_mask(0x09, 0xE0), Some(0x80));
        assert_eq!(allowed_change_mask(0x09, 0xAA), None);
        assert_eq!(allowed_change_mask(0x0B, 0xF7), Some(0x80));
        assert_eq!(allowed_change_mask(0x01, 0x02), None);
    }
}
