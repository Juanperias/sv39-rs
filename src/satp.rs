#[inline(always)]
pub fn csrr_satp() -> u64 {
    let mut value: u64;
    unsafe {
        ::core::arch::asm!("csrr {}, satp", out(reg) value);
    }
    value
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Satp {
    pub mode: PagingMode,
    pub asid: u16,
    pub ppn: u64,
}

impl Satp {
    pub fn read() -> Self {
        let satp = csrr_satp();

        let mode = (satp >> 60) & 0xF;
        let asid = (satp >> 44) & 0xFFFF;
        let ppn = satp & 0xFFFFFFFFFFF;

        Self {
            mode: PagingMode::from(mode as u8),
            asid: asid as u16,
            ppn,
        }
    }

    pub fn write() {
        todo!()
    }
}


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PagingMode {
    Bare,
    Sv39,
    Unk(u8),
}

impl From<u8> for PagingMode {
    fn from(value: u8) -> Self {
        match value {
             0 => Self::Bare,
             8 => Self::Sv39,
             _ => Self::Unk(value),
        }
    }
}

impl Into<u8> for PagingMode {
    fn into(self) -> u8 {
        match self {
            PagingMode::Bare => 0,
            PagingMode::Sv39 => 8,
            PagingMode::Unk(v) => v,
        }
    }
}
