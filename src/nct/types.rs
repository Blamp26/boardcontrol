#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ldn(pub u8);

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NctRegister(pub u8);

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RmwStep {
    pub ldn: Ldn,
    pub reg: NctRegister,
    pub and_mask: u8,
    pub or_mask: u8,
}
