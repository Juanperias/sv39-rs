// Code inspired in x86_64 lib

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    error::Sv39Error,
    page::{Page, PhysFrame, alloc::PhysFrameAllocator},
    paging::{PageFlags, PageTable, PageTableEntry, PhysAddr, VirtAddr},
    tlb,
};

pub trait Mapper {
    /// Maps a Virtual Page to a Physical Page with Flags
    /// In most cases, you'll want PageFlags to contain the V flag
    fn map_to<T: PhysFrameAllocator>(
        &self,
        page: Page,
        phys: PhysFrame,

        flags: PageFlags,
        level_2_table: &PhysFrame,
        allocator: &mut T,
    ) -> Result<TlbFlush, Sv39Error>;

    /// Maps a Range from a start page to an end page to a range of an start phys frame to an end phys frame
    /// If flush_tlb is true it will perform a Tlb Flush per map
    fn map_range<T: PhysFrameAllocator>(
        &self,
        start_page: Page,
        end_page: Page,
        start_phys: PhysFrame,
        end_phys: PhysFrame,
        flush_tlb: bool,

        flags: PageFlags,
        level_2_table: &PhysFrame,
        allocator: &mut T,
    ) -> Result<(), Sv39Error>;

    /// Unmaps a page, IT DOES NOT DEALLOCATE
    fn umap(&self, page: Page, level_2_table: &PhysFrame) -> Result<TlbFlush, Sv39Error>;

    /// Walks the 3-level page table and returns a mutable reference to the leaf PageTableEntry
    fn page_walk_mut<'a>(&self, virt: VirtAddr, level_2_table: &PhysFrame) -> Result<&'a mut PageTableEntry, Sv39Error>;
    
    /// Walks the 3-level page table and returns a reference to the leaf PageTableEntry
    fn page_walk<'a>(&self, virt: VirtAddr, level_2_table: &PhysFrame) -> Result<&'a PageTableEntry, Sv39Error>;
}

/// Create a map with an offset
/// Dev Note: It's a bit ambiguous—basically, to access the PTEs located on pages, you're given the physical address. The problem is that when paging is enabled, it's generally not a VA = PA mapping. If it's a standard kernel, it will be in HHDM; in that case, the offset will be the higher-half offset (If you use limine you can get de higher half offset with a request).
/// If you’re starting from scratch and paging isn’t enabled, you’ll have to configure it yourself; therefore, the offset will be 0 (which tells the mapper that VA = PA). Once paging is configured, you should change the offset to the higher half (if your kernel is HHDM, which is most likely).
pub struct OffsetMapper(AtomicUsize);

/// NOTE: Initial Kernel Page setup should be done manually
/// But it isn't to hard  just make a OffsetMapper with offset 0
/// Map your kernel in HHDM and change de offset to HHDM
impl OffsetMapper {
    pub const fn new(offset: usize) -> Self {
        Self(AtomicUsize::new(offset))
    }

    /// Creates an offset mapper with offset 0
    pub const fn identity() -> Self {
        Self(AtomicUsize::new(0))
    }

    /// Set offset of a mapper
    pub fn set_offset(&self, offset: usize) {
        self.0.store(offset, Ordering::SeqCst);
    }
}

impl Mapper for OffsetMapper {
    fn map_to<T: PhysFrameAllocator>(
        &self,
        page: Page,
        phys: PhysFrame,

        flags: PageFlags,
        root_table_phys: &PhysFrame,
        allocator: &mut T,
    ) -> Result<TlbFlush, Sv39Error> {
        let off = self.0.load(Ordering::SeqCst);

        let v = page.start_address_virt();

        let root_table = unsafe {
            PageTable::from_ptr(
                (root_table_phys.start_address_phys().offset(off)?.addr()) as *mut PageTable,
            )
        };

        // This unwrap should be safe, because a VPN is always 9 bits, i think in some point I
        // can change this to an .ok_or but this unwrap dont seem very unsafe
        let entry_2 = root_table.entry_mut(v.vpn_2() as usize).unwrap();

        if !entry_2.is_valid() {
            let p = allocator.alloc().ok_or(Sv39Error::AllocationFailed)?;
            let table_virt = p.start_address_phys().offset(off)?;
            unsafe {
                core::ptr::write_bytes(table_virt.addr() as *mut u8, 0, 4096);
            }

            entry_2.set_flags(PageFlags::V);
            entry_2.set_phys(p.start_address_phys().clone());
        }

        let level_1_page = entry_2.phys_addr().offset(off)?;
        let level_1_table = unsafe { PageTable::from_ptr(level_1_page.addr() as *mut PageTable) };

        let entry_1 = level_1_table.entry_mut(v.vpn_1() as usize).unwrap();

        if !entry_1.is_valid() {
            let p = allocator.alloc().ok_or(Sv39Error::AllocationFailed)?;
            let table_virt = p.start_address_phys().offset(off)?;
            unsafe {
                core::ptr::write_bytes(table_virt.addr() as *mut u8, 0, 4096);
            }

            entry_1.set_flags(PageFlags::V);
            entry_1.set_phys(p.start_address_phys().clone());
        }

        let level_0_page = entry_1.phys_addr().offset(off)?;
        let level_0_table = unsafe { PageTable::from_ptr(level_0_page.addr() as *mut PageTable) };

        let entry_0 = level_0_table.entry_mut(v.vpn_0() as usize).unwrap();

        entry_0.set_flags(flags);
        entry_0.set_phys(phys.start_address_phys().clone());

        Ok(TlbFlush(v.clone()))
    }

    fn map_range<T: PhysFrameAllocator>(
        &self,
        start_page: Page,
        end_page: Page,

        start_phys: PhysFrame,
        end_phys: PhysFrame,

        flush_tlb: bool,

        flags: PageFlags,
        level_2_table: &PhysFrame,
        allocator: &mut T,
    ) -> Result<(), Sv39Error> {
        for (virt, phys) in ((start_page.start_address()..end_page.start_address()).step_by(4096))
            .zip((start_phys.start_address()..end_phys.start_address()).step_by(4096))
        {
            let tlb = self.map_to(
                // NOTE: This should be safe because safe_page and end_page SHOULD be aligned and we are making 4 KIB states
                Page::new(unsafe { VirtAddr::new_unchecked(virt) }),
                PhysFrame::new(unsafe { PhysAddr::new_unchecked(phys) }),
                flags,
                level_2_table,
                allocator,
            )?;

            if flush_tlb {
                tlb.flush()
            }
        }

        Ok(())
    }

    fn umap(&self, page: Page, level_2_table: &PhysFrame) -> Result<TlbFlush, Sv39Error> {
        let start_v = page.start_address_virt();

        let off = self.0.load(Ordering::SeqCst);

        let root_table = unsafe {
            PageTable::from_ptr(
                (level_2_table.start_address_phys().offset(off)?.addr()) as *mut PageTable,
            )
        };

        let entry_2 = root_table.entry(start_v.vpn_2() as usize).unwrap();

        if !entry_2.is_valid() {
            return Err(Sv39Error::InvalidPte);
        }

        let level_1_page = entry_2.phys_addr().offset(off)?;
        let level_1_table = unsafe { PageTable::from_ptr(level_1_page.addr() as *mut PageTable) };

        let entry_1 = level_1_table.entry(start_v.vpn_1() as usize).unwrap();

        if !entry_1.is_valid() {
            return Err(Sv39Error::InvalidPte);
        }

        let level_0_page = entry_1.phys_addr().offset(off)?;
        let level_0_table = unsafe { PageTable::from_ptr(level_0_page.addr() as *mut PageTable) };

        let entry_0 = level_0_table.entry_mut(start_v.vpn_0() as usize).unwrap();

        *entry_0 = PageTableEntry::empty();

        Ok(TlbFlush(start_v.clone()))
    }

    fn page_walk<'a>(&self, virt: VirtAddr, level_2_table: &PhysFrame) -> Result<&'a PageTableEntry, Sv39Error> {
        let off = self.0.load(Ordering::SeqCst);

        let root_table = unsafe {
            PageTable::from_ptr(
                (level_2_table.start_address_phys().offset(off)?.addr()) as *mut PageTable,
            )
        };

        let entry_2 = root_table.entry(virt.vpn_2() as usize).ok_or(Sv39Error::InvalidPte)?;
        if !entry_2.is_valid() {
            return Err(Sv39Error::InvalidPte);
        }

        let level_1_page = entry_2.phys_addr().offset(off)?;
        let level_1_table = unsafe { PageTable::from_ptr(level_1_page.addr() as *mut PageTable) };

        let entry_1 = level_1_table.entry(virt.vpn_1() as usize).ok_or(Sv39Error::InvalidPte)?;

        if !entry_1.is_valid() {
            return Err(Sv39Error::InvalidPte);
        }

        let level_0_page = entry_1.phys_addr().offset(off)?;
        let level_0_table = unsafe { PageTable::from_ptr(level_0_page.addr() as *mut PageTable) };

        let entry_0 = level_0_table.entry(virt.vpn_0() as usize).ok_or(Sv39Error::InvalidPte)?;
        
        if !entry_0.is_valid() {
            return Err(Sv39Error::InvalidPte);
        }

        Ok(entry_0)
    }

    
    fn page_walk_mut<'a>(&self, virt: VirtAddr, level_2_table: &PhysFrame) -> Result<&'a mut PageTableEntry, Sv39Error> {
        let off = self.0.load(Ordering::SeqCst);

        let root_table = unsafe {
            PageTable::from_ptr(
                (level_2_table.start_address_phys().offset(off)?.addr()) as *mut PageTable,
            )
        };

        let entry_2 = root_table.entry(virt.vpn_2() as usize).ok_or(Sv39Error::InvalidPte)?;

        if !entry_2.is_valid() {
            return Err(Sv39Error::InvalidPte);
        }

        let level_1_page = entry_2.phys_addr().offset(off)?;
        let level_1_table = unsafe { PageTable::from_ptr(level_1_page.addr() as *mut PageTable) };

        let entry_1 = level_1_table.entry(virt.vpn_1() as usize).ok_or(Sv39Error::InvalidPte)?;

        if !entry_1.is_valid() {
            return Err(Sv39Error::InvalidPte);
        }

        let level_0_page = entry_1.phys_addr().offset(off)?;
        let level_0_table = unsafe { PageTable::from_ptr(level_0_page.addr() as *mut PageTable) };

        let entry_0 = level_0_table.entry_mut(virt.vpn_0() as usize).ok_or(Sv39Error::InvalidPte)?;
        
        if !entry_0.is_valid() {
            return Err(Sv39Error::InvalidPte);
        }

        Ok(entry_0)
    }
}

/// TLB handler; it is obtained after each mapping 
pub struct TlbFlush(VirtAddr);

impl TlbFlush {
    /// Flush the specified address WITHOUT ASID
    pub fn flush(self) {
        tlb::clear_virt(self.0);
    }

    /// Flush the specified address with ASID
    pub fn flush_with_asid(self, asid: u16) {
        tlb::clear_virt_asid(self.0, asid);
    }

    /// Nothing....
    pub fn ignore(self) {}
}
