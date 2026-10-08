use sv39::{page::{PhysFrame, alloc::PhysFrameAllocator}, paging::PhysAddr};

unsafe extern "C" {
    static mut __heap: u8;
    static mut __heap_end: u8;
}

static mut OFFSET: usize = 0;

pub struct PageAllocator;

impl PhysFrameAllocator for PageAllocator {
    fn alloc(&self) -> Option<PhysFrame> {
        let heap_start = (&raw mut __heap).addr();
    let heap_end = (&raw mut __heap_end).addr();

    unsafe {
        let ret = heap_start + OFFSET;

        OFFSET += 4096;

        if heap_start + OFFSET > heap_end {
            panic!("OOM");
        }

        Some(PhysFrame::new(PhysAddr::new( ret as u64).unwrap()))
    }
    }

    fn free(&self, p: PhysFrame) {}
}
