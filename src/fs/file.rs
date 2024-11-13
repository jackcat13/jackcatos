use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::disk::disk::Disk;

use super::{fat::fat16::{fat16_init, FatPrivate, ResolveError}, path_parser::PathPart};

pub const MAX_OS_FILESYSTEMS: usize = 12;
pub const MAX_OS_FILEDESCRIPTORS: usize = 512;

pub enum FileSeekMode {
    SEEK_SET,
    SEEK_CUR,
    SEEK_END,
}

pub enum FileMode {
    READ,
    WRITE,
    APPEND,
    INVALID,
}

pub struct File {
    
}

type FsOpen = fn(Box<Disk>, &PathPart, &FileMode) -> Option<u8>;
type FsResolve = fn(Disk) -> Result<Box<FatPrivate>, ResolveError>;

#[derive(Debug, Clone, Copy)]
pub struct FileSystem {
    pub resolve: FsResolve,
    pub open: FsOpen,
    
    pub name: [char; 20],
}

lazy_static! {
    pub static ref FILE_SYSTEMS: Mutex<[Option<FileSystem>; MAX_OS_FILESYSTEMS]> = Mutex::new([None; MAX_OS_FILESYSTEMS]);
}

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub index: u32,
    pub filesystem: Box<FileSystem>,
    pub disk: Box<Disk>,
}

lazy_static! {
    pub static ref FILE_DESCRIPTORS: Mutex<Vec<Option<FileDescriptor>>> = Mutex::new(Vec::new());
}

pub fn fs_init() {
    fs_static_load(); 
}

pub fn fopen(filename: String, mode: String) {
    
}

pub fn fs_insert_fs(fs: FileSystem) {
    for i in 0..MAX_OS_FILESYSTEMS {
        if FILE_SYSTEMS.lock()[i].is_none() {
            FILE_SYSTEMS.lock()[i] = Some(fs);
            return;
        }
    }
    panic!("No more filesystems available!");
}

pub fn fs_static_load() {
    fs_insert_fs(fat16_init());
}

pub fn fs_resolve(disk: Arc<Mutex<Disk>>) -> Option<u8> {
    for i in 0..MAX_OS_FILESYSTEMS {
        if let Some(fs) = FILE_SYSTEMS.lock()[i] {
            let mut disk = disk.lock();
            let fat_private = (fs.resolve)(disk.clone());
            if fat_private.is_ok() {
                disk.fat_private = Some(fat_private.unwrap());
                disk.filesystem = Some(fs);
                return Some(0);
            }
        }
    }
    None
}