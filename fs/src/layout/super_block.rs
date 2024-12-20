#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap: u32,
    pub inode_blocks: u32,
    pub data_bitmap: u32,
    pub data_blocks: u32,
}

const SUPER_BLOCK_MAGIC: u32 = 0x78887C6A;

impl SuperBlock {
    pub fn init_with(
        &mut self,
        total_blocks: u32,
        inode_bitmap: u32,
        inode_blocks: u32,
        data_bitmap: u32,
        data_blocks: u32,
    ) {
        *self = Self {
            magic: SUPER_BLOCK_MAGIC,
            total_blocks,
            inode_bitmap,
            inode_blocks,
            data_bitmap,
            data_blocks,
        }
    }

    pub fn check(&self) -> bool {
        self.magic == SUPER_BLOCK_MAGIC
    }
}
