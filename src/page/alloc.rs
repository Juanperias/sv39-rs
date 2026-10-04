use crate::page::PhysFrame;

pub trait PhysFrameAllocator {
    pub fn alloc(&self) -> Option<PhysFrame>;
    pub fn free(&self, frame: PhysFrame);
}
