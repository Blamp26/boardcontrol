use std::collections::HashMap;

use crate::backend::Backend;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEvent {
    Read { ldn: u8, reg: u8, value: u8 },
    Write { ldn: u8, reg: u8, value: u8 },
    Delay { ms: u64 },
}

#[derive(Debug, Default, Clone)]
pub struct TraceBackend {
    registers: HashMap<(u8, u8), u8>,
    log: Vec<TraceEvent>,
}

impl TraceBackend {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn set_reg(&mut self, ldn: u8, reg: u8, value: u8) {
        self.registers.insert((ldn, reg), value);
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn reg(&self, ldn: u8, reg: u8) -> Option<u8> {
        self.registers.get(&(ldn, reg)).copied()
    }

    pub fn log(&self) -> &[TraceEvent] {
        &self.log
    }
}

impl Backend for TraceBackend {
    fn read_ldn_reg(&mut self, ldn: u8, reg: u8) -> Result<u8> {
        let value = self.registers.get(&(ldn, reg)).copied().unwrap_or(0);
        self.log.push(TraceEvent::Read { ldn, reg, value });
        Ok(value)
    }

    fn write_ldn_reg(&mut self, ldn: u8, reg: u8, value: u8) -> Result<()> {
        self.registers.insert((ldn, reg), value);
        self.log.push(TraceEvent::Write { ldn, reg, value });
        Ok(())
    }

    fn delay_ms(&mut self, ms: u64) -> Result<()> {
        self.log.push(TraceEvent::Delay { ms });
        Ok(())
    }
}
