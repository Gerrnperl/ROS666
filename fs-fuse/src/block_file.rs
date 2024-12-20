use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    sync::Mutex,
};

use fs::block_dev::BlockDevice;

pub const BLOCK_SIZE: usize = 512;

pub struct BlockFile(pub Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, offset: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((offset * BLOCK_SIZE) as u64))
            .unwrap();
        file.read(buf).unwrap();
    }

    fn write_block(&self, offset: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((offset * BLOCK_SIZE) as u64))
            .unwrap();
        file.write(buf).unwrap();
    }
}
