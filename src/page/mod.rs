use crate::paging::{PhysAddr, VirtAddr};

pub mod alloc;

// TODO(Juanperias): manage more than 4 KIB pages

pub struct Page(VirtAddr);

impl Page {
    pub fn new(addr: VirtAddr) -> Self {
        Self(addr)
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.0.addr() as *mut u8
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.0.addr() as *const u8
    }
    
}

pub struct PhysFrame(PhysAddr);

impl PhysFrame {
    pub fn new(phys: PhysAddr) -> Self {
        Self(phys)
    }
}

/*
pub trait PageSize {
    const SIZE: usize;
}

pub struct Size4Kib;

impl PageSize for Size4Kib {
   const SIZE: usize = 4096;
}

pub struct Size2Mib;

impl PageSize for Size2Mib {
    const SIZE: usize = 2 << 20;
}


pub struct Size1Gib;

impl PageSize for Size1Gib {
    const SIZE: usize = 1 << 30;
}

*/
