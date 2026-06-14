use crate::backend::Backend;
use crate::error::{Error, Result};

use super::allowlist::allowed_change_mask;
use super::rmw::safe_rmw;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NctRmwOp {
    pub ldn: u8,
    pub reg: u8,
    pub and_mask: u8,
    pub or_mask: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NctOp {
    Rmw(NctRmwOp),
    Delay(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NctSequence {
    ops: Vec<NctOp>,
}

impl NctSequence {
    pub fn new(ops: Vec<NctOp>) -> Self {
        Self { ops }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn ops(&self) -> &[NctOp] {
        &self.ops
    }

    pub fn execute<B: Backend>(&self, backend: &mut B) -> Result<()> {
        for op in &self.ops {
            match *op {
                NctOp::Rmw(rmw) => {
                    let allowed = allowed_change_mask(rmw.ldn, rmw.reg).ok_or(
                        Error::MissingAllowlistEntry {
                            ldn: rmw.ldn,
                            reg: rmw.reg,
                        },
                    )?;
                    safe_rmw(
                        backend,
                        rmw.ldn,
                        rmw.reg,
                        rmw.and_mask,
                        rmw.or_mask,
                        allowed,
                    )?;
                }
                NctOp::Delay(ms) => {
                    backend.delay_ms(ms)?;
                }
            }
        }
        Ok(())
    }
}

pub fn init_sequence_7a45() -> NctSequence {
    NctSequence::new(vec![
        NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0xE0,
            and_mask: 0x7F,
            or_mask: 0x00,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0xE9,
            and_mask: 0xFF,
            or_mask: 0x80,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0x27,
            and_mask: 0xEF,
            or_mask: 0x00,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0x1B,
            and_mask: 0xBF,
            or_mask: 0x00,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x0B,
            reg: 0xF7,
            and_mask: 0xFF,
            or_mask: 0x80,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0xE0,
            and_mask: 0xFF,
            or_mask: 0x80,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0x30,
            and_mask: 0xFF,
            or_mask: 0x02,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0x2A,
            and_mask: 0xFF,
            or_mask: 0x40,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x08,
            reg: 0xF0,
            and_mask: 0x7F,
            or_mask: 0x00,
        }),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x08,
            reg: 0xF1,
            and_mask: 0xFF,
            or_mask: 0x80,
        }),
    ])
}

pub fn reset_led_sequence_7a45() -> NctSequence {
    NctSequence::new(vec![
        NctOp::Rmw(NctRmwOp {
            ldn: 0x0B,
            reg: 0xF7,
            and_mask: 0x7F,
            or_mask: 0x00,
        }),
        NctOp::Delay(10),
        NctOp::Rmw(NctRmwOp {
            ldn: 0x0B,
            reg: 0xF7,
            and_mask: 0xFF,
            or_mask: 0x80,
        }),
    ])
}

#[cfg(test)]
mod tests {
    use crate::backend::trace::{TraceBackend, TraceEvent};
    use crate::error::Error;

    use super::{NctOp, init_sequence_7a45, reset_led_sequence_7a45};

    #[test]
    fn test_7a45_init_sequence_has_10_steps() {
        let sequence = init_sequence_7a45();
        assert_eq!(sequence.ops().len(), 10);
    }

    #[test]
    fn test_7a45_reset_sequence() {
        let sequence = reset_led_sequence_7a45();
        assert_eq!(sequence.ops().len(), 3);
        assert!(matches!(
            sequence.ops()[0],
            NctOp::Rmw(op) if op.ldn == 0x0B && op.reg == 0xF7 && op.and_mask == 0x7F && op.or_mask == 0x00
        ));
        assert!(matches!(sequence.ops()[1], NctOp::Delay(10)));
        assert!(matches!(
            sequence.ops()[2],
            NctOp::Rmw(op) if op.ldn == 0x0B && op.reg == 0xF7 && op.and_mask == 0xFF && op.or_mask == 0x80
        ));
    }

    #[test]
    fn test_no_blind_write_api_for_nct_sequence() {
        let sequence = init_sequence_7a45();
        assert!(
            sequence
                .ops()
                .iter()
                .all(|op| matches!(op, NctOp::Rmw(_) | NctOp::Delay(_)))
        );
    }

    #[test]
    fn test_execute_7a45_init_sequence_trace_backend() {
        let mut backend = TraceBackend::new();

        let result = init_sequence_7a45().execute(&mut backend);
        assert!(result.is_ok());
        assert_eq!(backend.log().len(), 20);

        let reads = backend
            .log()
            .iter()
            .filter(|event| matches!(event, TraceEvent::Read { .. }))
            .count();
        let writes = backend
            .log()
            .iter()
            .filter(|event| matches!(event, TraceEvent::Write { .. }))
            .count();

        assert_eq!(reads, 10);
        assert_eq!(writes, 10);
        assert_eq!(backend.reg(0x09, 0xE0), Some(0x80));
        assert_eq!(backend.reg(0x09, 0xE9), Some(0x80));
        assert_eq!(backend.reg(0x0B, 0xF7), Some(0x80));
        assert_eq!(backend.reg(0x09, 0x30), Some(0x02));
        assert_eq!(backend.reg(0x09, 0x2A), Some(0x40));
        assert_eq!(backend.reg(0x08, 0xF1), Some(0x80));
    }

    #[test]
    fn test_execute_7a45_reset_sequence_trace_backend() {
        let mut backend = TraceBackend::new();
        backend.set_reg(0x0B, 0xF7, 0x80);

        let result = reset_led_sequence_7a45().execute(&mut backend);
        assert!(result.is_ok());
        assert_eq!(
            backend.log(),
            &[
                TraceEvent::Read {
                    ldn: 0x0B,
                    reg: 0xF7,
                    value: 0x80,
                },
                TraceEvent::Write {
                    ldn: 0x0B,
                    reg: 0xF7,
                    value: 0x00,
                },
                TraceEvent::Delay { ms: 10 },
                TraceEvent::Read {
                    ldn: 0x0B,
                    reg: 0xF7,
                    value: 0x00,
                },
                TraceEvent::Write {
                    ldn: 0x0B,
                    reg: 0xF7,
                    value: 0x80,
                },
            ]
        );
        assert_eq!(backend.reg(0x0B, 0xF7), Some(0x80));
    }

    #[test]
    fn test_execute_sequence_missing_allowlist_blocks() {
        let mut backend = TraceBackend::new();
        let sequence = super::NctSequence::new(vec![super::NctOp::Rmw(super::NctRmwOp {
            ldn: 0x09,
            reg: 0xAA,
            and_mask: 0xFF,
            or_mask: 0x01,
        })]);

        let result = sequence.execute(&mut backend);
        assert!(matches!(
            result,
            Err(Error::MissingAllowlistEntry {
                ldn: 0x09,
                reg: 0xAA,
            })
        ));
        assert_eq!(backend.log().len(), 0);
    }
}
