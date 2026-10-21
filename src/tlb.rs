use crate::paging::VirtAddr;

#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
/// Flushes all entries from the TLB
pub extern "C" fn clear_all() {
    core::arch::naked_asm!("sfence.vma zero, zero", "ret");
}

#[cfg(not(target_arch = "riscv64"))]
pub extern "C" fn clear_all() {}

#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
/// Flushes all entries with THAT asid
pub extern "C" fn clear_asid(asid: u16) {
    core::arch::naked_asm!("sfence.vma zero, a0", "ret");
}

#[cfg(not(target_arch = "riscv64"))]
pub extern "C" fn clear_asid(_asid: u16) {}

#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
/// Flushes all entries with THAT virtual address
pub extern "C" fn clear_virt(v: VirtAddr) {
    core::arch::naked_asm!("sfence.vma a0, zero", "ret");
}

#[cfg(not(target_arch = "riscv64"))]
pub extern "C" fn clear_virt(_v: VirtAddr) {}

#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
/// Flushes all entries with THAT virtual address and ASID
pub extern "C" fn clear_virt_asid(v: VirtAddr, asid: u16) {
    core::arch::naked_asm!("sfence.vma a0, a1", "ret");
}

#[cfg(not(target_arch = "riscv64"))]
pub extern "C" fn clear_virt_asid(_v: VirtAddr, _asid: u16) {}
