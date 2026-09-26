unsafe extern "C" {
    static mut __heap: u8;
    static mut __heap_end: u8;
}

static mut OFFSET: usize = 0;

pub fn alloc_page() -> *mut u8 {
    let heap_start = (&raw mut __heap).addr();
    let heap_end = (&raw mut __heap_end).addr();

    unsafe {
        let ret = heap_start + OFFSET;

        OFFSET += 4096;

        if heap_start + OFFSET > heap_end {
            panic!("OOM");
        }

        ret as *mut u8
    }
}
