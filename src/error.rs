use thiserror::Error;

#[derive(Error, Debug)]
pub enum Sv39Error {
    #[error("Misaligned Address: 0x{0:X}")]
    MisalignedAddr(u64),

    #[error("Invalid Address: 0x{0:X}")]
    InvalidAddr(u64),
}
