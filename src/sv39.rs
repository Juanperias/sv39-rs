#[derive(Debug)]
#[repr(align(4096))]
pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    pub fn new() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }
}

#[derive(Debug)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn ext(&self) -> Ext {
        let napot = (self.0 >> 63) & 1;

        let pbmt = (self.0 >> 61) & 3;

        Ext {
            pbmt: pbmt as u8,
            napot: napot as u8,
        }
    }
    pub fn ppn_2(&self) -> usize {
        ((self.0 >> 28) & 0x3FFFFFF) as usize
    }
    pub fn ppn_1(&self) -> usize {
        ((self.0 >> 19) & 0x1FF) as usize
    }
    pub fn ppn_0(&self) -> usize {
        ((self.0 >> 10) & 0x1FF) as usize
    }
    pub fn rsw(&self) -> u8 {
        ((self.0 >> 8) & 3) as u8
    }
    pub fn flags(&self) {
        let flags = ((self.0) & 0xFF) as u8;

        PageFlags::from_bits_retain(flags);
    }
}

#[derive(Debug)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub fn ppn_2(&self) -> usize {
        ((self.0 >> 30) & 0x3FFFFFF) as usize
    }
    pub fn ppn_1(&self) -> usize {
        ((self.0 >> 21) & 0x1FF) as usize
    }
    pub fn ppn_0(&self) -> usize {
        ((self.0 >> 9) & 0x1FF) as usize
    }
    pub fn page_offset(&self) -> usize {
        (self.0 & 0xFFF) as usize
    }
}

#[derive(Debug)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    pub fn vpn_2(&self) -> usize {
        ((self.0 >> 30) & 0x1FF) as usize
    }
    
    pub fn vpn_1(&self) -> usize {
        ((self.0 >> 21) & 0x1FF) as usize
    }

    pub fn vpn_0(&self) -> usize {
        ((self.0 >> 12) & 0x1FF) as usize
    }

    pub fn page_offset(&self) -> usize {
        (self.0 & 0xFFF) as usize
    }
}



#[derive(Debug)]
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
