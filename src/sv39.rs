use core::arch::asm;

use crate::{error::Sv39Error, satp::Satp};

#[derive(Debug)]
#[repr(align(4096))]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    pub const fn empty() -> Self {
        Self([const { PageTableEntry::empty() }; 512])
    }

    pub unsafe fn from_ptr<'a>(ptr: *mut PageTable) -> &'a mut PageTable {
        unsafe { &mut *(ptr) }
    }

    pub const fn as_ptr(&self) -> *const PageTableEntry {
        self.0.as_ptr()
    }

    pub const fn as_mut_ptr(&mut self) -> *mut PageTableEntry {
        self.0.as_mut_ptr()
    }

    pub fn load_with_phys(&self, p_addr: PhysAddr, asid: u16) {
        let satp = Satp {
            mode: crate::satp::PagingMode::Sv39,
            asid,
            ppn: p_addr.0 >> 12,
        };

        satp.write_csr();

        unsafe {
            // TODO(Juanperias): Impl selective sfence.vma
            asm!(
                "sfence.vma"
            );
        }

    }

    pub fn entry(&self, num: usize) -> Option<&PageTableEntry> {
        self.0.get(num)
    }

    pub fn entry_mut(&mut self, num: usize) -> Option<&mut PageTableEntry> {
        self.0.get_mut(num)
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn new(phys: PhysAddr, rsw: u8, flags: PageFlags) -> PageTableEntry {
        let inner = 
            (phys.ppn_2() << 28) |
            (phys.ppn_1() << 19) |
            (phys.ppn_0() << 10) |
            (((rsw as u64) & 0x3) << 8) |
            (flags.bits() as u64) & 0xFF;

        PageTableEntry(inner)
    }

    pub fn with_ext() {
        todo!()
    }

    pub const fn empty() -> PageTableEntry {
        PageTableEntry(0)
    }

    pub fn set_phys(&mut self, addr: PhysAddr) {
        let inner_ppn = 
            (addr.ppn_2() << 28) |
            (addr.ppn_1() << 19) |
            (addr.ppn_0() << 10);

        self.0 = (self.0 & 0xFFC00000000003FF) | inner_ppn;
    }

    pub fn set_flags(&mut self, flags: PageFlags) {
        let inner = (self.0 & 0xFFFFFFFFFFFFFF00) | (flags.bits() & 0xFF) as u64;
    
        self.0 = inner;
    }

    pub fn ext(&self) -> Ext {
        let napot = (self.0 >> 63) & 1;

        let pbmt = (self.0 >> 61) & 3;

        Ext {
            pbmt: pbmt as u8,
            napot: napot as u8,
        }
    }
    pub fn ppn_2(&self) -> u64 {
        (self.0 >> 28) & 0x3FFFFFF
    }
    pub fn ppn_1(&self) -> u64 {
        (self.0 >> 19) & 0x1FF
    }
    pub fn ppn_0(&self) -> u64 {
        (self.0 >> 10) & 0x1FF
    }
    pub fn rsw(&self) -> u8 {
        ((self.0 >> 8) & 3) as u8
    }
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PageFlags::V)
    }
    pub fn flags(&self) -> PageFlags {
        let flags = ((self.0) & 0xFF) as u8;

        PageFlags::from_bits_retain(flags)
    }
}

#[derive(Debug)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub fn new(addr: u64) -> Result<PhysAddr, Sv39Error> {
        if (addr & 0xFFF) != 0 {
          return Err(Sv39Error::MisalignedAddr(addr));
        }

        Ok(PhysAddr(addr))
    }

    pub unsafe fn new_unchecked(addr: u64) -> PhysAddr {
        PhysAddr(addr)
    }

    pub unsafe fn from_parts(ppn_2: u64, ppn_1: u64, ppn_0: u64, page_offset: u16) -> PhysAddr {
        let inner = 
            (ppn_2 << 30) |
            (ppn_1 << 21) |
            (ppn_0 << 12) |
            ((page_offset as u64) & 0xFFF);

        PhysAddr(inner)
    }
    
    pub fn ppn_2(&self) -> u64 {
        (self.0 >> 30) & 0x3FFFFFF
    }
    pub fn ppn_1(&self) -> u64 {
        (self.0 >> 21) & 0x1FF
    }
    pub fn ppn_0(&self) -> u64 {
        (self.0 >> 9) & 0x1FF
    }
    pub fn page_offset(&self) -> u64 {
        self.0 & 0xFFF
    }
}

#[derive(Debug)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    pub fn new(addr: u64) -> Result<Self, Sv39Error> {
         if (addr & 0xFFF) != 0 {
            return Err(Sv39Error::MisalignedAddr(addr));
         }

         
         if (((addr as i64) << 25) >> 25) != addr as i64 {
            return Err(Sv39Error::InvalidAddr(addr));
         }


         Ok(Self(addr))
    }

    pub unsafe fn new_unchecked(addr: u64) -> Self {
        Self(addr)
    }
    
    pub fn vpn_2(&self) -> u64 {
        (self.0 >> 30) & 0x1FF
    }
    
    pub fn vpn_1(&self) -> u64 {
        (self.0 >> 21) & 0x1FF
    }

    pub fn vpn_0(&self) -> u64 {
        (self.0 >> 12) & 0x1FF
    }

    pub fn page_offset(&self) -> u64 {
        self.0 & 0xFFF
    }
}



#[derive(Debug, Default)]
pub struct Ext {
    napot: u8,
    pbmt: u8,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PageFlags: u8 {
        const V = 0x0;
        const R = 0x1;
        const W = 0x3;
        const X = 0x7;
        const U = 0xF;
        const G = 0x1F;
        const A = 0x3f;
        const D = 0x7F;
    }
}
