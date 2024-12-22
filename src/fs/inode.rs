use alloc::{sync::Arc, vec::Vec};
use common::syscall::OpenFlags;
use lazy_static::lazy_static;
use ros_fs::{fs::FileSystemRootInode, layout::disk_inode::InodeType, virt_fs::MemInode};
use spin::Mutex;

use crate::{drivers::block::BLOCK_DEVICE, mm::page_table::UserBuffer};

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

    pub fn read_all(&self) -> Vec<u8> {
        let mut inode = self.inode.lock();
        let inode = &mut *inode;
        let mut buf = Vec::new();
        let _size = inode.inode.get_size();
        loop {
            let mut data = [0u8; 4096];
            let size = inode.inode.read_at(inode.offset, &mut data);
            if size == 0 {
                break;
            }
            buf.extend_from_slice(&data[..size]);
            inode.offset += size;
        }
        buf
    }
}

impl File for OSInode {
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inode = self.inode.lock();
        let inode = &mut *inode;
        let mut read_size = 0;
        for i in 0..buf.buffers.len() {
            let newly_read = inode.inode.read_at(inode.offset, buf.buffers[i]);
            if newly_read == 0 {
                break;
            }
            inode.offset += newly_read;
            read_size += newly_read;
        }
        read_size
    }

    fn write(&self, buf: UserBuffer) -> usize {
        let mut inode = self.inode.lock();
        let inode = &mut *inode;
        let mut write_size = 0;
        for i in 0..buf.buffers.len() {
            let newly_written = inode.inode.write_at(inode.offset, buf.buffers[i]);
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

pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let readable = flags.readable();
    let writable = flags.writable();
    if flags.contains(OpenFlags::CREATE) {
        return create_file(name, flags);
    }
    let inode = ROOT_INODE.find(name)?;
    if flags.contains(OpenFlags::TRUNCATE) {
        inode.clear();
    }
    Some(Arc::new(OSInode::new(readable, writable, inode)))
}

fn create_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let readable = flags.readable();
    let writable = flags.writable();
    if let Some(inode) = ROOT_INODE.find(name) {
        inode.clear();
        return Some(Arc::new(OSInode::new(readable, writable, inode)));
    }
    let inode = ROOT_INODE.create(name, InodeType::File);
    Some(Arc::new(OSInode::new(readable, writable, inode)))
}
