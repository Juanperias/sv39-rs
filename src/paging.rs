use crate::{error::Sv39Error, page::PhysFrame, satp::Satp, tlb};

/// It represents a PageTable with its 512 entries 
#[derive(Debug)]
#[repr(align(4096))]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    /// Creates an empty (zeroed) page table
    pub const fn empty() -> Self {
        Self([const { PageTableEntry::empty() }; 512])
    }
    
    pub unsafe fn from_ptr<'a>(ptr: *mut PageTable) -> &'a mut PageTable {
        unsafe { &mut *ptr }
    }

    pub const fn as_ptr(&self) -> *const PageTableEntry {
        self.0.as_ptr()
    }

    pub const fn as_mut_ptr(&mut self) -> *mut PageTableEntry {
        self.0.as_mut_ptr()
    }

    pub fn entry(&self, num: usize) -> Option<&PageTableEntry> {
        self.0.get(num)
    }

    pub fn entry_mut(&mut self, num: usize) -> Option<&mut PageTableEntry> {
        self.0.get_mut(num)
    }
}

/// I load a physical address (where the page table should ideally be located) into the Satp CSR. If the asid field is NULL, it will invalidate the entire TLB; if it is not NULL, it will invalidate all entries with that asid.
/// The logic behind assigning and reassigning ASIDs is completely out of scope of this lib
pub fn load_with_phys(p_addr: &PhysFrame, asid: Option<u16>) {
    let satp = Satp {
        mode: PagingMode::Sv39,
        asid: asid.unwrap_or(0),
        ppn: p_addr.start_address() >> 12,
    };

    satp.write_csr();

    match asid {
        Some(v) => {
            tlb::clear_asid(v);
        }
        None => {
            tlb::clear_all();
        }
    }
}

/// Page table entry; for a complete definition, see: <https://docs.riscv.org/reference/isa/v20260120/priv/supervisor.html#sv39> Figure 22
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Creates a new PTE without extensions (bits 63-61 zeroed)
    pub fn new(phys: PhysAddr, rsw: u8, flags: PageFlags) -> PageTableEntry {
        let inner = (phys.ppn_2() << 28)
            | (phys.ppn_1() << 19)
            | (phys.ppn_0() << 10)
            | (((rsw as u64) & 0x3) << 8)
            | (flags.bits() as u64) & 0xFF;

        PageTableEntry(inner)
    }

    /// Creates a new PTE with extensions
    pub fn with_ext(ext: Ext, phys: PhysAddr, rsw: u8, flags: PageFlags) -> PageTableEntry {
        let napot = (ext.napot & 1) as u64;
        let pbmt = (ext.pbmt & 3) as u64;

        let inner = napot << 63
            | pbmt << 61
            | (phys.ppn_2() << 28)
            | (phys.ppn_1() << 19)
            | (phys.ppn_0() << 10)
            | (((rsw as u64) & 0x3) << 8)
            | (flags.bits() as u64) & 0xFF;

        PageTableEntry(inner)
    }

    pub const fn empty() -> PageTableEntry {
        PageTableEntry(0)
    }

    /// Set the phys address to the PTE (Must aligned to 4096
    pub fn set_phys(&mut self, addr: PhysAddr) {
        let ppn = addr.0 >> 12;

        self.0 = (self.0 & 0xFFC00000000003FF) | (ppn << 10);
    }
   
    /// Set the PTE flags
    pub fn set_flags(&mut self, flags: PageFlags) {
        let inner = (self.0 & 0xFFFFFFFFFFFFFF00) | (flags.bits() & 0xFF) as u64;

        self.0 = inner;
    }

    /// Get extension from PTE
    pub fn ext(&self) -> Ext {
        let napot = (self.0 >> 63) & 1;

        let pbmt = (self.0 >> 61) & 3;

        Ext {
            pbmt: pbmt as u8,
            napot: napot as u8,
        }
    }

    /// Get physical address from PTE
    pub fn phys_addr(&self) -> PhysAddr {
        unsafe { PhysAddr::from_parts(self.ppn_2(), self.ppn_1(), self.ppn_0(), 0) }
    }

    /// Get Physical Page Number 2
    pub fn ppn_2(&self) -> u64 {
        (self.0 >> 28) & 0x3FFFFFF
    }

    /// Get Physical Page Number 1
    pub fn ppn_1(&self) -> u64 {
        (self.0 >> 19) & 0x1FF
    }

    /// Get Physical Page Number 0
    pub fn ppn_0(&self) -> u64 {
        (self.0 >> 10) & 0x1FF
    }

    /// Reserved For Software (The Kernel, Cowboy)
    pub fn rsw(&self) -> u8 {
        ((self.0 >> 8) & 3) as u8
    }

    /// Check if page contains V flag
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PageFlags::V)
    }

    /// Get PageFlags
    pub fn flags(&self) -> PageFlags {
        let flags = ((self.0) & 0xFF) as u8;

        PageFlags::from_bits_retain(flags)
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
/// Physical Address representation, for more information  visit <https://docs.riscv.org/reference/isa/v20260120/priv/supervisor.html#sv39> Figure 21
pub struct PhysAddr(u64);

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

    pub fn offset(&self, offset: usize) -> Result<PhysAddr, Sv39Error> {
        Self::new(self.0 + (offset as u64))
    }

    pub const fn addr(&self) -> u64 {
        self.0
    }

    pub unsafe fn from_parts(ppn_2: u64, ppn_1: u64, ppn_0: u64, page_offset: u16) -> PhysAddr {
        let inner = (ppn_2 << 30) | (ppn_1 << 21) | (ppn_0 << 12) | ((page_offset as u64) & 0xFFF);

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

#[derive(Debug, Clone)]
#[repr(transparent)]
/// Virtual Address, for more information visit <https://docs.riscv.org/reference/isa/v20260120/priv/supervisor.html#sv39> Figure 20
pub struct VirtAddr(u64);

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

    pub fn offset(&self, offset: usize) -> Result<VirtAddr, Sv39Error> {
        Self::new(self.0 + (offset as u64))
    }

    pub const fn addr(&self) -> u64 {
        self.0
    }

    /// Virtual Page Number 2
    pub fn vpn_2(&self) -> u64 {
        (self.0 >> 30) & 0x1FF
    }

    /// Virtual Page Number 1
    pub fn vpn_1(&self) -> u64 {
        (self.0 >> 21) & 0x1FF
    }

    /// Virtual Page Number 0
    pub fn vpn_0(&self) -> u64 {
        (self.0 >> 12) & 0x1FF
    }

    /// Page Offset from 0-4096
    pub fn page_offset(&self) -> u64 {
        self.0 & 0xFFF
    }
}

#[derive(Debug, Default)]
/// Page Extensions, for more information visit: <https://docs.riscv.org/reference/isa/v20260120/priv/supervisor.html#svnapot> and <https://docs.riscv.org/reference/isa/v20260120/priv/supervisor.html#svpbmt>
pub struct Ext {
    napot: u8,
    pbmt: u8,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// Page Flags, It is the 0-7 bits from the PTE
    pub struct PageFlags: u8 {
        /// Valid Page. If you are mapping a Virtual Address this should be in the PTE
        const V = 1 << 0;
        /// Readable Page
        const R = 1 << 1;
        /// Writable Page
        const W = 1 << 2;
        /// Executable Page
        const X = 1 << 3;
        /// This page is User-Accesible
        const U = 1 << 4;
        /// Global page
        const G = 1 << 5;
        /// Accessed Page
        const A = 1 << 6;
        /// Dirty Flag
        const D = 1 << 7;
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PagingMode {
    Bare,
    Sv39,
    Unk(u8),
}

impl From<u8> for PagingMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Bare,
            8 => Self::Sv39,
            _ => Self::Unk(value),
        }
    }
}

impl Into<u8> for PagingMode {
    fn into(self) -> u8 {
        match self {
            PagingMode::Bare => 0,
            PagingMode::Sv39 => 8,
            PagingMode::Unk(v) => v,
        }
    }
}
