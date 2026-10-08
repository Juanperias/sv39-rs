#![no_std]
#![no_main]

use sv39::{page::{Page, PhysFrame, alloc::PhysFrameAllocator, mapper::Mapper}, paging::{PageFlags, PageTable, PhysAddr, VirtAddr}};

pub mod alloc;

unsafe extern "C" {
    static mut __bss: u8;
    static mut __bss_end: u8;
    static mut __heap: u8;
    static mut __heap_end: u8;
    static mut __kernel_base: u8;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub extern "C" fn boot() -> ! {
    unsafe {
        core::arch::asm!("la sp, __stack_top",);

        core::arch::asm!("csrw stvec, {}", in(reg) trap_handler as *const () as usize);

        let bss_start = &raw mut __bss;
        let bss_size = (&raw mut __bss_end as usize) - (&raw mut __bss as usize);
        core::ptr::write_bytes(bss_start, 0, bss_size);

        let mut allocator = alloc::PageAllocator;
        
        let p = allocator.alloc().unwrap().start_address_phys().addr();

        let level_2_page_table = PageTable::from_ptr(p as *mut PageTable);

        let kernel_base = (&raw mut __kernel_base) as u64;
        let heap_end = (&raw mut __heap_end) as u64; // OR KERNEL_END

        let offset_mapper = sv39::page::mapper::OffsetMapper::identity();

        for i in (kernel_base..heap_end).step_by(4096) {
            offset_mapper.map_to(
                Page::new(VirtAddr::new(i).unwrap()),
                PhysFrame::new(PhysAddr::new(i).unwrap()),
                PageFlags::V | PageFlags::W | PageFlags::R | PageFlags::X,
                level_2_page_table,
                &mut allocator,
            ).unwrap();
        }

        println!("Loading paging");

       level_2_page_table.load_with_phys(PhysAddr::new(p).unwrap(), 0);

       println!("Paging loaded! 1:1 mapping is here");

       core::arch::asm!("j main", in("a0") level_2_page_table, options(noreturn));
    }
}



#[unsafe(no_mangle)]
pub extern "C" fn main(level_2_page_table: *mut PageTable) {
    println!("Welcome to dummy kernel!");
    println!("Level 2 Page Table is in {:?}", level_2_page_table);

    loop {}
}

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            sbi_putchar(byte);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = writeln!($crate::Printer, $($arg)*);
    }};
}

pub fn sbi_putchar(ch: u8) {
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a6") 0,
            in("a7") 1,
            inout("a0") ch as usize => _,
            out("a1") _
        );
    }
}

macro_rules! read_csr {
    ($csr:expr) => {{
        let mut value: u64;
        unsafe {
            ::core::arch::asm!(concat!("csrr {}, ", $csr), out(reg) value);
        }
        value
    }};
}

#[unsafe(link_section = ".text.stvec")]
pub fn trap_handler() -> ! {
    let scause = read_csr!("scause");
    let sepc = read_csr!("sepc");
    let stval = read_csr!("stval");
    let scause_str = match scause {
        0 => "instruction address misaligned",
        1 => "instruction access fault",
        2 => "illegal instruction",
        3 => "breakpoint",
        4 => "load address misaligned",
        5 => "load access fault",
        6 => "store/AMO address misaligned",
        7 => "store/AMO access fault",
        8 => "environment call from U/VU-mode",
        9 => "environment call from HS-mode",
        10 => "environment call from VS-mode",
        11 => "environment call from M-mode",
        12 => "instruction page fault",
        13 => "load page fault",
        15 => "store/AMO page fault",
        20 => "instruction guest-page fault",
        21 => "load guest-page fault",
        22 => "virtual instruction",
        23 => "store/AMO guest-page fault",
        0x8000_0000_0000_0000 => "user software interrupt",
        0x8000_0000_0000_0001 => "supervisor software interrupt",
        0x8000_0000_0000_0002 => "hypervisor software interrupt",
        0x8000_0000_0000_0003 => "machine software interrupt",
        0x8000_0000_0000_0004 => "user timer interrupt",
        0x8000_0000_0000_0005 => "supervisor timer interrupt",
        0x8000_0000_0000_0006 => "hypervisor timer interrupt",
        0x8000_0000_0000_0007 => "machine timer interrupt",
        0x8000_0000_0000_0008 => "user external interrupt",
        0x8000_0000_0000_0009 => "supervisor external interrupt",
        0x8000_0000_0000_000a => "hypervisor external interrupt",
        0x8000_0000_0000_000b => "machine external interrupt",
        _ => panic!("unknown scause: {:#x}", scause),
    };

    panic!(
        "trap handler: {} at {:#x} (stval={:#x})",
        scause_str, sepc, stval
    );
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("{:?}", info);

    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
