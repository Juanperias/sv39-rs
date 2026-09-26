use crate::paging::PagingMode;

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn csrr_satp() -> u64 {
    core::arch::naked_asm!("csrr a0, satp", "ret");
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn csrw_satp(val: u64) {
    core::arch::naked_asm!("csrw satp, a0", "ret",);
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

    pub fn encode(&self) -> u64 {
        let mode: u8 = self.mode.clone().into();

        let mode = mode as u64;

        let satp =
            ((mode & 0xF) << 60) | (((self.asid as u64) & 0xFFFF) << 44) | self.ppn & 0xFFFFFFFFFFF;

        satp
    }

    pub fn write_csr(&self) {
        csrw_satp(self.encode());
    }
}
