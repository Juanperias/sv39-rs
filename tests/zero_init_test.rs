use sv39::{
    page::{
        alloc::PhysFrameAllocator,
        mapper::{Mapper, OffsetMapper},
        Page, PhysFrame,
    },
    paging::{PageFlags, PageTable, PhysAddr, VirtAddr},
};

#[repr(align(4096))]
    #[allow(dead_code)]
    struct DirtyFrame([u8; 4096]);

struct DirtyFrameAllocator {
    frames: [core::cell::RefCell<DirtyFrame>; 4],
    next: core::cell::Cell<usize>,
}

impl DirtyFrameAllocator {
    fn new() -> Self {
        Self {
            // Fill with non-zero dirty memory (0xAA) to simulate recycled pages
            frames: core::array::from_fn(|_| core::cell::RefCell::new(DirtyFrame([0xAA; 4096]))),
            next: core::cell::Cell::new(0),
        }
    }
}

impl PhysFrameAllocator for DirtyFrameAllocator {
    fn alloc(&self) -> Option<PhysFrame> {
        let idx = self.next.get();
        if idx >= self.frames.len() {
            return None;
        }
        self.next.set(idx + 1);
        let ptr = self.frames[idx].as_ptr() as *const u8 as u64;
        Some(PhysFrame::new(PhysAddr::new(ptr).unwrap()))
    }

    fn free(&self, _frame: PhysFrame) {}
}

#[test]
fn test_newly_allocated_tables_are_zeroed() {
    let mut allocator = DirtyFrameAllocator::new();
    let root_frame = allocator.alloc().expect("Root table frame");
    unsafe {
        core::ptr::write_bytes(root_frame.start_address() as *mut u8, 0, 4096);
    }

    let mapper = OffsetMapper::identity();
    let v_addr = VirtAddr::new(0x2000_0000).unwrap();
    let p_addr = PhysAddr::new(0x8000_0000).unwrap();

    let _tlb = mapper
        .map_to(
            Page::new(v_addr.clone()),
            PhysFrame::new(p_addr),
            PageFlags::V | PageFlags::R | PageFlags::W,
            &root_frame,
            &mut allocator,
        )
        .expect("Mapping succeeds");

    // Inspect the root L2 table
    let root_table = unsafe { PageTable::from_ptr(root_frame.start_address() as *mut PageTable) };
    let l2_unmapped = root_table.entry(1).expect("Entry 1");
    assert!(
        !l2_unmapped.is_valid(),
        "Non-mapped entry 1 in root table must not have V flag set"
    );

    // Unmapped entries in freshly allocated L1 and L0 tables must also have is_valid == false
    let l2_target = root_table.entry(v_addr.vpn_2() as usize).unwrap();
    assert!(l2_target.is_valid());

    let l1_table = unsafe { PageTable::from_ptr(l2_target.phys_addr().addr() as *mut PageTable) };
    // Adjacent entry in L1 table should be clean zero, NOT 0xAA
    let l1_adjacent = l1_table.entry((v_addr.vpn_1() as usize + 1) % 512).unwrap();
    assert!(
        !l1_adjacent.is_valid(),
        "Adjacent entry in L1 table must be zeroed, not dirty 0xAA"
    );
}
