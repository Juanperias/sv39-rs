use crate::page::PhysFrame;

/// Physical frame (4 kib pages... probably in the future 2 mib and 1 gib pages?) allocator
pub trait PhysFrameAllocator {
    /// Allocate a valid 4 kib aligned physical frame.
    fn alloc(&self) -> Option<PhysFrame>;
    /// Free a valid 4 kib aligned physical frame
    fn free(&self, frame: PhysFrame);
}
