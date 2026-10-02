use core::marker::PhantomData;

use crate::error::Sv39Error;

pub mod alloc;

pub struct Page<T: PageSize> {
    ptr: *mut u8,
    _size: PhantomData<T>,
}

impl<T: PageSize> Page<T> {
    pub fn new(ptr: *mut u8) -> Result<Self, Sv39Error> {
        if ptr.align_offset(T::SIZE) == 0 {
            return Ok(Self {
                ptr,
                _size: PhantomData,
            });
        }

        return Err(Sv39Error::MisalignedAddr(ptr.addr() as u64))
    }

    pub unsafe fn new_unchecked(ptr: *mut u8) -> Self {
        Self { ptr, _size: PhantomData }
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr as *const u8
    }
    
}

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
