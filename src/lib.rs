#![no_std]


#[cfg(
    not(
        any(
            target_arch = "riscv64",
            target_pointer_width = "64",    
        )
    )
)]
compile_error!("SV39 only has support for RISCV 64");


pub mod error;
pub mod sv39;
pub mod satp;
