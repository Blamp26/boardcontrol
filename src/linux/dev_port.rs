use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::error::{Error, Result};
use crate::nct::superio::RawPortIo;

pub struct DevPort {
    file: File,
}

impl DevPort {
    pub fn open() -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/port")
            .map_err(|err| Error::DevPortOpenFailed(err.to_string()))?;
        Ok(Self { file })
    }

    pub fn read_u8(&mut self, port: u16) -> Result<u8> {
        self.file
            .seek(SeekFrom::Start(u64::from(port)))
            .map_err(|err| Error::DevPortIoFailed(err.to_string()))?;
        let mut buf = [0u8; 1];
        self.file
            .read_exact(&mut buf)
            .map_err(|err| Error::DevPortIoFailed(err.to_string()))?;
        Ok(buf[0])
    }

    pub fn write_u8(&mut self, port: u16, value: u8) -> Result<()> {
        self.file
            .seek(SeekFrom::Start(u64::from(port)))
            .map_err(|err| Error::DevPortIoFailed(err.to_string()))?;
        self.file
            .write_all(&[value])
            .map_err(|err| Error::DevPortIoFailed(err.to_string()))
    }
}

impl RawPortIo for DevPort {
    fn read_u8(&mut self, port: u16) -> Result<u8> {
        DevPort::read_u8(self, port)
    }

    fn write_u8(&mut self, port: u16, value: u8) -> Result<()> {
        DevPort::write_u8(self, port, value)
    }
}

#[cfg(test)]
mod tests {
    use crate::nct::superio::SUPERIO_INDEX_PORT;

    #[test]
    fn dev_port_trait_constants_compile() {
        let _ = SUPERIO_INDEX_PORT;
    }
}
