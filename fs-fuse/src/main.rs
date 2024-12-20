//! # 文件系统镜像打包工具
//!
//! 用于将应用程序打包成文件系统镜像，以便于加载到系统内核中。
//!
//! ## 使用方法
//!
//! ```text
//! Usage: fs-fuse [OPTIONS] --target <TARGET> --output <OUTPUT>
//!
//! Options:
//!   -a, --app <APP>...     应用程序名称列表，以空格分隔
//!   -t, --target <TARGET>  目标目录，存放应用程序二进制文件
//!   -o, --output <OUTPUT>  文件系统镜像输出路径
//!   -h, --help             Print help (see more with '--help')
//!   -V, --version          Print version
//! ```
//!
use std::{
    fs::{File, OpenOptions},
    io::Read,
    sync::{Arc, Mutex},
};

use block_file::BlockFile;
use clap::Parser;
use fs::{
    fs::{FileSystem, FileSystemRootInode},
    layout::disk_inode::InodeType,
};

mod block_file;

/// 应用程序打包工具
///
/// 用于将应用程序打包成文件系统镜像，以便于加载到系统内核中。
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, help = "应用程序名称列表，以空格分隔", value_parser, num_args = 1.., value_delimiter = ' ')]
    app: Vec<String>,
    #[arg(short, long, help = "目标目录，存放应用程序二进制文件")]
    target: String,
    #[arg(short, long, help = "文件系统镜像输出路径")]
    output: String,
}

pub fn main() {
    let args = Args::parse();
    fs_pack(args).expect("打包应用程序失败");
}

fn fs_pack(args: Args) -> std::io::Result<()> {
    let mut apps_path = args.target;
    if !apps_path.ends_with('/') {
        apps_path.push('/');
    }
    let app_paths = args
        .app
        .into_iter()
        .map(|app| (app.clone(), format!("{}{}", apps_path, app)))
        .collect::<Vec<_>>();
    let mut fsimg_path = args.output;
    if fsimg_path.ends_with('/') {
        fsimg_path.push_str("fs.img");
    }

    println!(
        "\x1b[1;32m[INFO]\x1b[0m 正在打包应用程序 {} -> {}",
        apps_path, fsimg_path
    );

    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(fsimg_path)?;
        f.set_len(16 * 2048 * 512).unwrap();
        f
    })));
    let fs = FileSystem::new(block_file, 16 * 2048, 1);
    let root_inode = Arc::new(fs.root_inode());

    for (app, path) in app_paths.iter() {
        println!("\x1b[1;32m[INFO]\x1b[0m 装载应用程序：{}", path);
        let mut file_data: Vec<u8> = Vec::new();
        File::open(path.as_str())
            .unwrap()
            .read_to_end(&mut file_data)
            .unwrap();
        let inode = root_inode.create(app.as_str(), InodeType::File);
        inode.write_at(0, file_data.as_slice());
    }

    println!("\x1b[1;32m[INFO]\x1b[0m 正在校验应用程序");

    let ls = root_inode.ls();
    assert_eq!(ls.len(), app_paths.len());

    for (app, path) in app_paths.iter() {
        print!("\x1b[1;32m[INFO]\x1b[0m 校验应用程序：{}\t", path);
        let mut src_file_data: Vec<u8> = Vec::new();
        File::open(path.as_str())
            .unwrap()
            .read_to_end(&mut src_file_data)
            .unwrap();
        let inode = root_inode.find(app.as_str()).unwrap();

        let src_size = src_file_data.len();
        let dst_size = inode.get_size() as usize;
        assert_eq!(src_size, dst_size);

        let mut dst_file_data: Vec<u8> = vec![0; dst_size];
        inode.read_at(0, &mut dst_file_data);

        for i in 0..src_size {
            assert_eq!(src_file_data[i], dst_file_data[i],);
        }
        println!("\x1b[0;32m通过\x1b[0m");
    }
    Ok(())
}
