// Code inspired in x86_64 lib

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{error::Sv39Error, page::{Page, PhysFrame, alloc::PhysFrameAllocator}, paging::{PageFlags, PageTable, PhysAddr, VirtAddr}};

pub trait Mapper {
    fn map_to<T: PhysFrameAllocator>(
        &self,
        page: Page,
        phys: PhysFrame,

        flags: PageFlags,
        level_2_table: &PhysFrame,
        allocator: &mut T,
) -> Result<(), Sv39Error>;

    fn umap(
        &self,
        page: Page,
        level_2_table: &PhysFrame,
    ) -> Result<(), Sv39Error>;
}

pub struct OffsetMapper(AtomicUsize);

/// NOTE: Initial Kernel Page setup should be done manually
/// But it isn't to hard  just make a OffsetMapper with offset 0
/// Map your kernel in HHDM and change de offset to HHDM
impl OffsetMapper {
    pub const fn new(offset: usize) -> Self {
        Self(AtomicUsize::new(offset))
    }

    pub const fn identity() -> Self {
        Self(AtomicUsize::new(0))
    }
    
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
    ) -> Result<(), Sv39Error> {
        let off = self.0.load(Ordering::SeqCst);
        
        let v = page.start_address_virt();


        let root_table = unsafe { PageTable::from_ptr((root_table_phys.start_address_phys().offset(off)?.addr()) as *mut PageTable) };
            
        // This unwrap should be safe, because a VPN is always 9 bits, i think in some point I
        // can change this to an .ok_or but this unwrap dont seem very unsafe
        let entry_2 = root_table.entry_mut(v.vpn_2() as usize).unwrap();

        if !entry_2.is_valid() {
            let p = allocator.alloc()
                .ok_or(Sv39Error::AllocationFailed)?;

            entry_2.set_flags(PageFlags::V);
            entry_2.set_phys(p.start_address_phys().clone()); 
        }

        let level_1_page = entry_2.phys_addr().offset(off)?;
        let level_1_table = unsafe { PageTable::from_ptr(level_1_page.addr() as *mut PageTable) };

        let entry_1 = level_1_table.entry_mut(v.vpn_1() as usize).unwrap();

        if !entry_1.is_valid() {
            let p = allocator.alloc()
                .ok_or(Sv39Error::AllocationFailed)?;
            
            entry_1.set_flags(PageFlags::V);
            entry_1.set_phys(p.start_address_phys().clone());
        }

        let level_0_page = entry_1.phys_addr().offset(off)?;
        let level_0_table = unsafe { PageTable::from_ptr(level_0_page.addr() as *mut PageTable) };

        let entry_0 = level_0_table.entry_mut(v.vpn_0() as usize).unwrap();

        entry_0.set_flags(flags);
        entry_0.set_phys(phys.start_address_phys().clone());
      
        Ok(())
    }

    fn umap(
        &self,
        page: Page,
        level_2_table: &PhysFrame,
    ) -> Result<(), Sv39Error> {
        todo!()
   }

    
}

