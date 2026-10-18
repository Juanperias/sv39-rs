use sv39::{
    page::{
        alloc::PhysFrameAllocator,
        mapper::{Mapper, OffsetMapper},
        Page, PhysFrame,
    },
    paging::{PageFlags, PhysAddr, VirtAddr},
};

#[repr(align(4096))]
#[allow(dead_code)]
struct MockFrame([u8; 4096]);

struct MockAllocator {
    frames: [core::cell::RefCell<MockFrame>; 4],
    next: core::cell::Cell<usize>,
}

impl MockAllocator {
    fn new() -> Self {
        Self {
            frames: core::array::from_fn(|_| core::cell::RefCell::new(MockFrame([0; 4096]))),
            next: core::cell::Cell::new(0),
        }
    }
}

impl PhysFrameAllocator for MockAllocator {
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
fn test_translate_addr_roundtrip() {
    let mut allocator = MockAllocator::new();
    let root_frame = allocator.alloc().expect("Root frame");

    let mapper = OffsetMapper::identity();
    let virt = VirtAddr::new(0x4000_0000).unwrap();
    let phys = PhysAddr::new(0x8020_0000).unwrap();

    // Mapping virtual 0x4000_0000 to physical 0x8020_0000
    let _ = mapper
        .map_to(
            Page::new(virt.clone()),
            PhysFrame::new(phys.clone()),
            PageFlags::V | PageFlags::R | PageFlags::W | PageFlags::X,
            &root_frame,
            &mut allocator,
        )
        .expect("Mapping succeeded");

    // Translation should resolve to exactly 0x8020_0000
    let translated = mapper
        .translate_addr(virt.clone(), &root_frame)
        .expect("Translation must succeed");
    assert_eq!(
        translated.addr(),
        phys.addr(),
        "Translated physical address must match mapped physical address"
    );

    // After unmapping, translation must fail
    mapper
        .umap(Page::new(virt.clone()), &root_frame)
        .expect("Unmapping succeeded");
    assert!(
        mapper.translate_addr(virt, &root_frame).is_err(),
        "Translation must fail after unmapping"
    );
}
