pub mod inode;

pub trait File: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> usize;
    fn write(&self, buf: &[u8]) -> usize;
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
}
