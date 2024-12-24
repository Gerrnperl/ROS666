//! 操作系统 Inode (文件) 结构
//!
//! 一个文件在操作系统中对应一个 Inode 结构，用于管理文件的读写操作
use alloc::{string::String, sync::Arc, vec::Vec};
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

/// OS Inode
///
/// 文件层级的 Inode 结构，对 内存 Inode [MemInode] 的文件读写访问的封装
///
/// 一个 文件 (OS Inode) 可以被多个进程共享，因此需要实现 Send 和 Sync trait
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
    /// 创建一个新的 OS Inode
    pub fn new(readable: bool, writable: bool, inode: Arc<MemInode>) -> Self {
        Self {
            readable,
            writable,
            inode: Mutex::new(InodeData { inode, offset: 0 }),
        }
    }

    /// 读取文件的所有内容
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
        if inode.inode.get_type() == InodeType::Dir {
            // a trick to read directory
            let ls_str = inode.inode.ls().join("\t");
            let ls_bytes = ls_str.as_bytes();
            let mut read_size = 0;
            for i in 0..buf.buffers.len() {
                let mut newly_read = 0;
                while newly_read < ls_bytes.len() && read_size < buf.buffers[i].len() {
                    buf.buffers[i][read_size] = ls_bytes[newly_read];
                    newly_read += 1;
                    read_size += 1;
                }
                if newly_read == 0 {
                    break;
                }
            }
            return read_size;
        }
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

/// 打开文件
///
/// 根据文件名和打开标志打开文件
///
/// ## 参数
/// - `name`：文件名
/// - `flags`：打开标志, 包括读写标志和创建标志等
///
/// ## 返回
/// 成功打开文件时返回文件的 OS Inode，否则返回 None
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    if name == "/" {
        return Some(Arc::new(OSInode::new(true, false, ROOT_INODE.clone())));
    }
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

/// 创建文件
///
/// 根据文件名和打开标志创建文件
///
/// 在文件存在时清空文件内容
///
/// ## 参数
/// - `name`：文件名
/// - `flags`：打开标志, 包括读写标志和创建标志等
///
/// ## 返回
/// 成功创建文件时返回文件的 OS Inode，否则返回 None
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
