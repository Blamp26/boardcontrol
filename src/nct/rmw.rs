use crate::backend::Backend;
use crate::error::{Error, Result};

pub fn safe_rmw<B: Backend>(
    backend: &mut B,
    ldn: u8,
    reg: u8,
    and_mask: u8,
    or_mask: u8,
    allowed_change_mask: u8,
) -> Result<()> {
    let current = backend.read_ldn_reg(ldn, reg)?;
    let new_value = (current & and_mask) | or_mask;
    let changed = current ^ new_value;

    if changed & !allowed_change_mask != 0 {
        return Err(Error::SequenceBlocked {
            ldn,
            reg,
            current,
            new_value,
            changed,
            allowed_change_mask,
        });
    }

    backend.write_ldn_reg(ldn, reg, new_value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::backend::trace::{TraceBackend, TraceEvent};

    use super::safe_rmw;

    #[test]
    fn allowlisted_bit_change_passes() {
        let mut backend = TraceBackend::new();
        backend.set_reg(0x09, 0xE0, 0x00);

        let result = safe_rmw(&mut backend, 0x09, 0xE0, 0x7F, 0x80, 0x80);
        assert!(result.is_ok());
        assert_eq!(backend.reg(0x09, 0xE0), Some(0x80));
        assert_eq!(
            backend.log(),
            &[
                TraceEvent::Read {
                    ldn: 0x09,
                    reg: 0xE0,
                    value: 0x00,
                },
                TraceEvent::Write {
                    ldn: 0x09,
                    reg: 0xE0,
                    value: 0x80,
                },
            ]
        );
    }

    #[test]
    fn non_allowlisted_bit_change_is_blocked() {
        let mut backend = TraceBackend::new();
        backend.set_reg(0x09, 0xE0, 0x00);

        let result = safe_rmw(&mut backend, 0x09, 0xE0, 0xFE, 0x01, 0x80);
        assert!(result.is_err());
        assert_eq!(backend.reg(0x09, 0xE0), Some(0x00));
        assert_eq!(
            backend.log(),
            &[TraceEvent::Read {
                ldn: 0x09,
                reg: 0xE0,
                value: 0x00,
            }]
        );
    }
}
