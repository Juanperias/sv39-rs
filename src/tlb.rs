use crate::paging::VirtAddr;

#[unsafe(naked)]
pub extern "C" fn clear_all() {
    core::arch::naked_asm!(
        "sfence.vma zero, zero",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "C" fn clear_asid(asid: u16) {
    core::arch::naked_asm!(
        "sfence.vma zero, a0",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "C" fn clear_virt(v: VirtAddr) {
    core::arch::naked_asm!(
        "sfence.vma a0, zero",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "C" fn clear_virt_asid(v: VirtAddr, asid: u16) {
    core::arch::naked_asm!(
        "sfence.vma a0, a1",
        "ret"
    );
}
