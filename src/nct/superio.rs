use crate::error::Result;

pub const SUPERIO_INDEX_PORT: u16 = 0x4E;
pub const SUPERIO_DATA_PORT: u16 = 0x4F;

pub trait RawPortIo {
    fn read_u8(&mut self, port: u16) -> Result<u8>;
    fn write_u8(&mut self, port: u16, value: u8) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NctChipId {
    pub id_high: u8,
    pub revision: u8,
}

impl NctChipId {
    pub fn is_nct6779d(&self) -> bool {
        self.id_high == 0xC5
    }
}

pub fn enter_config<P: RawPortIo>(io: &mut P) -> Result<()> {
    io.write_u8(SUPERIO_INDEX_PORT, 0x87)?;
    io.write_u8(SUPERIO_INDEX_PORT, 0x87)?;
    Ok(())
}

pub fn exit_config<P: RawPortIo>(io: &mut P) -> Result<()> {
    io.write_u8(SUPERIO_INDEX_PORT, 0xAA)?;
    Ok(())
}

pub fn read_global_reg<P: RawPortIo>(io: &mut P, reg: u8) -> Result<u8> {
    io.write_u8(SUPERIO_INDEX_PORT, reg)?;
    io.read_u8(SUPERIO_DATA_PORT)
}

pub fn read_nct6779d_chip_id<P: RawPortIo>(io: &mut P) -> Result<NctChipId> {
    enter_config(io)?;
    let result = (|| {
        let id_high = read_global_reg(io, 0x20)?;
        let revision = read_global_reg(io, 0x21)?;
        Ok(NctChipId { id_high, revision })
    })();
    let exit_result = exit_config(io);
    match (result, exit_result) {
        (Ok(chip_id), Ok(())) => Ok(chip_id),
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum PortEvent {
        Write { port: u16, value: u8 },
        Read { port: u16, value: u8 },
    }

    #[derive(Debug, Default)]
    struct FakeRawPortIo {
        log: Vec<PortEvent>,
        global_regs: HashMap<u8, u8>,
        selected_reg: Option<u8>,
    }

    impl FakeRawPortIo {
        fn set_global_reg(&mut self, reg: u8, value: u8) {
            self.global_regs.insert(reg, value);
        }

        fn log(&self) -> &[PortEvent] {
            &self.log
        }
    }

    impl RawPortIo for FakeRawPortIo {
        fn read_u8(&mut self, port: u16) -> Result<u8> {
            let value = if port == SUPERIO_DATA_PORT {
                self.selected_reg
                    .and_then(|reg| self.global_regs.get(&reg).copied())
                    .unwrap_or(0)
            } else {
                0
            };
            self.log.push(PortEvent::Read { port, value });
            Ok(value)
        }

        fn write_u8(&mut self, port: u16, value: u8) -> Result<()> {
            self.log.push(PortEvent::Write { port, value });
            if port == SUPERIO_INDEX_PORT {
                match value {
                    0x87 | 0xAA => {
                        if value == 0xAA {
                            self.selected_reg = None;
                        }
                    }
                    _ => self.selected_reg = Some(value),
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_enter_exit_config_sequence() {
        let mut io = FakeRawPortIo::default();
        enter_config(&mut io).unwrap();
        exit_config(&mut io).unwrap();

        assert_eq!(
            io.log(),
            &[
                PortEvent::Write {
                    port: SUPERIO_INDEX_PORT,
                    value: 0x87,
                },
                PortEvent::Write {
                    port: SUPERIO_INDEX_PORT,
                    value: 0x87,
                },
                PortEvent::Write {
                    port: SUPERIO_INDEX_PORT,
                    value: 0xAA,
                },
            ]
        );
    }

    #[test]
    fn test_read_global_reg_sequence() {
        let mut io = FakeRawPortIo::default();
        io.selected_reg = Some(0x20);
        io.set_global_reg(0x20, 0xC5);

        let value = read_global_reg(&mut io, 0x20).unwrap();
        assert_eq!(value, 0xC5);
        assert_eq!(
            io.log(),
            &[
                PortEvent::Write {
                    port: SUPERIO_INDEX_PORT,
                    value: 0x20,
                },
                PortEvent::Read {
                    port: SUPERIO_DATA_PORT,
                    value: 0xC5,
                },
            ]
        );
    }

    #[test]
    fn test_read_chip_id_sequence() {
        let mut io = FakeRawPortIo::default();
        io.set_global_reg(0x20, 0xC5);
        io.set_global_reg(0x21, 0x63);

        let chip_id = read_nct6779d_chip_id(&mut io).unwrap();
        assert_eq!(chip_id.id_high, 0xC5);
        assert_eq!(chip_id.revision, 0x63);
        assert!(chip_id.is_nct6779d());
    }

    #[test]
    fn test_non_nct6779d_chip_id() {
        let mut io = FakeRawPortIo::default();
        io.set_global_reg(0x20, 0xA1);
        io.set_global_reg(0x21, 0x63);

        let chip_id = read_nct6779d_chip_id(&mut io).unwrap();
        assert_eq!(chip_id.id_high, 0xA1);
        assert!(!chip_id.is_nct6779d());
    }
}
