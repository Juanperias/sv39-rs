#![no_std]

//! # Sv39 Rust Implementation
//! Sv39 is one of RISCV's paging systems, with 39-bit virtual addresses and a total address space of 512 GB.
//! ## Get Started!
//! One of the simplest examples is the dummy kernel found in the source code, which performs a 1:1 mapping without the higher half.
//! ```rust,ignore
//!  let kernel_base = (&raw mut __kernel_base) as u64;
//!  let heap_end = (&raw mut __heap_end) as u64; // OR KERNEL_END
//!  let offset_mapper = sv39::page::mapper::OffsetMapper::identity();
//!  offset_mapper
//!       .map_range(
//!                Page::new(VirtAddr::new(kernel_base).unwrap()),
//!                Page::new(VirtAddr::new(heap_end).unwrap()),
//!                PhysFrame::new(PhysAddr::new(kernel_base).unwrap()),
//!                PhysFrame::new(PhysAddr::new(heap_end).unwrap()),
//!                false,
//!                PageFlags::X | PageFlags::R | PageFlags::W | PageFlags::V,
//!                &level_2_page_table,
//!                &mut allocator,
//!            )
//!            .unwrap();
//!        println!("Loading paging");
//!        load_with_phys(&level_2_page_table, None);


#[cfg(not(target_arch = "riscv64"))]
compile_error!("SV39 only has support for RISCV-64");

pub mod error;
pub mod page;
pub mod paging;
pub mod satp;
pub mod tlb;
