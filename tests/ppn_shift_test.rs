use sv39::paging::{PageFlags, PageTableEntry, PhysAddr};

#[test]
fn test_ppn_0_extraction_and_pte_roundtrip() {
    // 0x1000 is page 1 (4096 bytes). PPN[0] MUST be 1.
    // In buggy code (self.0 >> 9), this returned 8.
    let pa = PhysAddr::new(0x1000).expect("Valid 4K physical address");
    assert_eq!(
        pa.ppn_0(),
        1,
        "PPN[0] must be 1 for address 0x1000, got {}",
        pa.ppn_0()
    );
    assert_eq!(pa.ppn_1(), 0);
    assert_eq!(pa.ppn_2(), 0);

    // Verify PageTableEntry roundtrip
    let flags = PageFlags::V | PageFlags::R | PageFlags::W;
    let pte = PageTableEntry::new(pa.clone(), 0, flags);
    assert_eq!(
        pte.phys_addr().addr(),
        pa.addr(),
        "PTE decoded physical address must match original"
    );
}
