use alloc::sync::Arc;
use lazy_static::lazy_static;
use ros_fs::{fs::FileSystemRootInode, virt_fs::MemInode};
use spin::Mutex;

use crate::drivers::block::BLOCK_DEVICE;

use super::File;

lazy_static! {
    pub static ref ROOT_INODE: Arc<MemInode> = Arc::new({
        let rosfs = ros_fs::fs::FileSystem::open(BLOCK_DEVICE.clone());
        let root_inode = rosfs.root_inode();
        root_inode
    });
}

pub struct OSInode {
    readable: bool,
    writable: bool,
    inode: Mutex<InodeData>,
}

pub struct InodeData {
    pub inode: Arc<MemInode>,
    pub offset: usize,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<MemInode>) -> Self {
        Self {
            readable,
            writable,
            inode: Mutex::new(InodeData { inode, offset: 0 }),
        }
    }
}

impl File for OSInode {
    fn read(&self, buf: &mut [u8]) -> usize {
        let mut inode = self.inode.lock();
        let inode = &mut *inode;
        let mut read_size = 0;
        for i in 0..buf.len() {
            let newly_read = inode.inode.read_at(inode.offset, &mut buf[i..]);
            if newly_read == 0 {
                break;
            }
            inode.offset += newly_read;
            read_size += newly_read;
        }
        read_size
    }

    fn write(&self, buf: &[u8]) -> usize {
        let mut inode = self.inode.lock();
        let inode = &mut *inode;
        let mut write_size = 0;
        for i in 0..buf.len() {
            let newly_written = inode.inode.write_at(inode.offset, &buf[i..]);
            if newly_written == 0 {
                break;
            }
            inode.offset += newly_written;
            write_size += newly_written;
        }
        write_size
    }

    fn readable(&self) -> bool {
        self.readable
    }

    fn writable(&self) -> bool {
        self.writable
    }
}
