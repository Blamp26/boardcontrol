use crate::nct::sequence::{NctSequence, init_sequence_7a45, reset_led_sequence_7a45};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile7A45;

impl Profile7A45 {
    pub fn new() -> Self {
        Self
    }

    pub fn init_sequence(&self) -> NctSequence {
        init_sequence_7a45()
    }

    pub fn reset_led_sequence(&self) -> NctSequence {
        reset_led_sequence_7a45()
    }
}

#[cfg(test)]
mod tests {
    use crate::board::profile_for;

    #[test]
    fn test_unknown_board_is_not_supported() {
        assert!(profile_for("1234").is_none());
        assert!(profile_for("7A45").is_some());
    }
}
