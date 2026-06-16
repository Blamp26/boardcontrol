use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidArgs(String),
    UnsupportedBoard(String),
    HostDmiMismatch(String),
    ProcIoportsReadFailed(String),
    HidInventoryReadFailed(String),
    DevPortOpenFailed(String),
    DevPortIoFailed(String),
    SequenceBlocked {
        ldn: u8,
        reg: u8,
        current: u8,
        new_value: u8,
        changed: u8,
        allowed_change_mask: u8,
    },
    MissingAllowlistEntry {
        ldn: u8,
        reg: u8,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidArgs(msg) => write!(f, "invalid args: {msg}"),
            Error::UnsupportedBoard(board) => write!(f, "unsupported board: {board}"),
            Error::HostDmiMismatch(msg) => write!(f, "{msg}"),
            Error::ProcIoportsReadFailed(msg) => {
                write!(f, "failed to read /proc/ioports: {msg}")
            }
            Error::HidInventoryReadFailed(msg) => {
                write!(f, "failed to read HID inventory metadata: {msg}")
            }
            Error::DevPortOpenFailed(msg) => write!(f, "failed to open /dev/port: {msg}"),
            Error::DevPortIoFailed(msg) => write!(f, "dev/port I/O error: {msg}"),
            Error::SequenceBlocked {
                ldn,
                reg,
                current,
                new_value,
                changed,
                allowed_change_mask,
            } => write!(
                f,
                "blocked RMW for LDN=0x{ldn:02X} REG=0x{reg:02X}: current=0x{current:02X} new=0x{new_value:02X} changed=0x{changed:02X} allowed=0x{allowed_change_mask:02X}"
            ),
            Error::MissingAllowlistEntry { ldn, reg } => {
                write!(
                    f,
                    "missing allowlist entry for LDN=0x{ldn:02X} REG=0x{reg:02X}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}
