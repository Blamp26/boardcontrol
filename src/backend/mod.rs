pub mod trace;

use crate::error::Result;

pub trait Backend {
    fn read_ldn_reg(&mut self, ldn: u8, reg: u8) -> Result<u8>;
    fn write_ldn_reg(&mut self, ldn: u8, reg: u8, value: u8) -> Result<()>;
    fn delay_ms(&mut self, ms: u64) -> Result<()>;
}
