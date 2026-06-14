use crate::backend::Backend;
use crate::error::{Error, Result};

use super::allowlist::allowed_change_mask;
use super::sequence::{NctOp, NctSequence};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RmwPlan {
    pub ldn: u8,
    pub reg: u8,
    pub current: u8,
    pub and_mask: u8,
    pub or_mask: u8,
    pub new_value: u8,
    pub changed: u8,
    pub allowed_change_mask: u8,
    pub allowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NctPlanStep {
    Rmw(RmwPlan),
    Delay(u64),
}

pub fn plan_sequence<B: Backend>(
    backend: &mut B,
    sequence: &NctSequence,
) -> Result<Vec<NctPlanStep>> {
    let mut steps = Vec::with_capacity(sequence.ops().len());

    for op in sequence.ops() {
        match *op {
            NctOp::Rmw(rmw) => {
                let allowed_change_mask =
                    allowed_change_mask(rmw.ldn, rmw.reg).ok_or(Error::MissingAllowlistEntry {
                        ldn: rmw.ldn,
                        reg: rmw.reg,
                    })?;
                let current = backend.read_ldn_reg(rmw.ldn, rmw.reg)?;
                let new_value = (current & rmw.and_mask) | rmw.or_mask;
                let changed = current ^ new_value;
                let allowed = changed & !allowed_change_mask == 0;
                steps.push(NctPlanStep::Rmw(RmwPlan {
                    ldn: rmw.ldn,
                    reg: rmw.reg,
                    current,
                    and_mask: rmw.and_mask,
                    or_mask: rmw.or_mask,
                    new_value,
                    changed,
                    allowed_change_mask,
                    allowed,
                }));
            }
            NctOp::Delay(ms) => {
                steps.push(NctPlanStep::Delay(ms));
            }
        }
    }

    Ok(steps)
}

#[cfg(test)]
mod tests {
    use crate::backend::trace::{TraceBackend, TraceEvent};

    use super::{NctPlanStep, plan_sequence};
    use crate::error::Error;
    use crate::nct::sequence::{
        NctOp, NctRmwOp, NctSequence, init_sequence_7a45, reset_led_sequence_7a45,
    };

    #[test]
    fn test_plan_rmw_allowed() {
        let mut backend = TraceBackend::new();
        backend.set_reg(0x09, 0xE0, 0x00);
        let sequence = NctSequence::new(vec![NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0xE0,
            and_mask: 0x7F,
            or_mask: 0x80,
        })]);

        let plan = plan_sequence(&mut backend, &sequence).unwrap();
        assert_eq!(plan.len(), 1);
        match plan[0] {
            NctPlanStep::Rmw(rmw) => {
                assert_eq!(rmw.current, 0x00);
                assert_eq!(rmw.new_value, 0x80);
                assert_eq!(rmw.changed, 0x80);
                assert!(rmw.allowed);
            }
            _ => panic!("expected rmw"),
        }
        assert_eq!(backend.reg(0x09, 0xE0), Some(0x00));
    }

    #[test]
    fn test_plan_rmw_disallowed() {
        let mut backend = TraceBackend::new();
        backend.set_reg(0x09, 0xE0, 0x00);
        let sequence = NctSequence::new(vec![NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0xE0,
            and_mask: 0xFE,
            or_mask: 0x01,
        })]);

        let plan = plan_sequence(&mut backend, &sequence).unwrap();
        match plan[0] {
            NctPlanStep::Rmw(rmw) => {
                assert!(!rmw.allowed);
                assert_eq!(rmw.allowed_change_mask, 0x80);
            }
            _ => panic!("expected rmw"),
        }
    }

    #[test]
    fn test_plan_missing_allowlist_errors() {
        let mut backend = TraceBackend::new();
        let sequence = NctSequence::new(vec![NctOp::Rmw(NctRmwOp {
            ldn: 0x09,
            reg: 0xAA,
            and_mask: 0xFF,
            or_mask: 0x01,
        })]);

        let result = plan_sequence(&mut backend, &sequence);
        assert!(matches!(
            result,
            Err(Error::MissingAllowlistEntry {
                ldn: 0x09,
                reg: 0xAA,
            })
        ));
        assert!(backend.log().is_empty());
    }

    #[test]
    fn test_plan_sequence_does_not_write() {
        let mut backend = TraceBackend::new();
        let plan = plan_sequence(&mut backend, &init_sequence_7a45()).unwrap();
        assert_eq!(plan.len(), 10);
        assert!(
            backend
                .log()
                .iter()
                .all(|event| matches!(event, TraceEvent::Read { .. }))
        );
        assert!(
            backend
                .log()
                .iter()
                .all(|event| !matches!(event, TraceEvent::Write { .. }))
        );
    }

    #[test]
    fn test_plan_reset_sequence_includes_delay() {
        let mut backend = TraceBackend::new();
        let plan = plan_sequence(&mut backend, &reset_led_sequence_7a45()).unwrap();
        assert_eq!(plan.len(), 3);
        assert!(matches!(plan[0], NctPlanStep::Rmw(_)));
        assert!(matches!(plan[1], NctPlanStep::Delay(10)));
        assert!(matches!(plan[2], NctPlanStep::Rmw(_)));
    }
}
