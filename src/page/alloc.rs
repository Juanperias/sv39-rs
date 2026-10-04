use crate::page::PhysFrame;

pub trait PhysFrameAllocator {
    fn alloc(&self) -> Option<PhysFrame>;
    fn free(&self, frame: PhysFrame);
}
