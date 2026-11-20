use crate::paging::PagingMode;

#[cfg(target_arch = "riscv64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
/// Raw read from SATP CSR
extern "C" fn csrr_satp() -> u64 {
    core::arch::naked_asm!("csrr a0, satp", "ret");
}

#[cfg(target_arch = "riscv64")]
#[unsafe(no_mangle)]
#[unsafe(naked)]
/// Raw write to SATP CSR
extern "C" fn csrw_satp(val: u64) {
    core::arch::naked_asm!("csrw satp, a0", "ret",);
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// Wrapper from SATP Csr
pub struct Satp {
    /// Current paging mode
    pub mode: PagingMode,
    /// Current ASID
    pub asid: u16,
    /// Physical Page Number to Level 2 Page Table or Current Page Table
    pub ppn: u64,
}

impl Satp {
    #[must_use]
    pub fn read() -> Self {
        let satp = csrr_satp();

        let mode = (satp >> 60) & 0xF;
        let asid = (satp >> 44) & 0xFFFF;
        let ppn = satp & 0x0FFF_FFFF_FFFF;

        Self {
            mode: PagingMode::from(mode as u8),
            asid: asid as u16,
            ppn,
        }
    }

    /// Encodes to a u64 (you can write by yourself using `csrw_satp` or `write_csr`)
    #[must_use]
    pub fn encode(&self) -> u64 {
        let mode: u8 = self.mode.clone().into();

        let mode = u64::from(mode);

        ((mode & 0xF) << 60) | ((u64::from(self.asid) & 0xFFFF) << 44) | self.ppn & 0x0FFF_FFFF_FFFF
    }

    /// Writes a Satp to the CSR
    pub fn write_csr(&self) {
        csrw_satp(self.encode());
    }
}
