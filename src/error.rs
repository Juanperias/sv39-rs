use thiserror::Error;

#[derive(Error)]
pub enum Sv39Error {
    #[error("Misaligned Address: 0x{:X}")]
    MisalignedAddr(u64),

    #[error("Invalid Address: 0x{:X}")]
    InvalidAddr(u64),
}
