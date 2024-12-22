const NAME_LEN_MAX: usize = 27;
const NAME_SIZE_MAX: usize = NAME_LEN_MAX + 1;
pub const DIR_ENTRY_SIZE: usize = core::mem::size_of::<DirEntry>();

/// 目录项
///
/// 一个目录项包含一个 inode 编号和一个文件名
///
/// 存储于目录 Inode 中
#[repr(C)]
pub struct DirEntry {
    pub inode: u32,
    pub name: [u8; NAME_SIZE_MAX],
}

impl DirEntry {
    /// 创建一个新的目录项
    pub fn new(inode: u32, name: &str) -> Self {
        let mut name_bytes = [0; NAME_SIZE_MAX];
        name.as_bytes().iter().enumerate().for_each(|(i, &b)| {
            name_bytes[i] = b;
        });
        Self {
            inode,
            name: name_bytes,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, DIR_ENTRY_SIZE) }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, DIR_ENTRY_SIZE) }
    }

    /// 获取目录项的文件名
    pub fn name(&self) -> &str {
        let end = self
            .name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(NAME_LEN_MAX);
        core::str::from_utf8(&self.name[..end]).unwrap()
    }

    /// 获取目录项的 inode 编号
    pub fn inode(&self) -> u32 {
        self.inode
    }
}

impl Default for DirEntry {
    fn default() -> Self {
        Self {
            inode: 0,
            name: [0; NAME_SIZE_MAX],
        }
    }
}
