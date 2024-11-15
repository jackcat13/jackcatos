use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{disk::disk::{get_disk, Disk}, println};

use super::{fat::fat16::{fat16_init, FatFileDescriptor, FatPrivate, ResolveError}, path_parser::{self, PathPart}};

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

type FsOpen = fn(&Disk, &PathPart, &FileMode) -> Result<(), ()>;
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
    pub filesystem: FileSystem,
    pub disk: Disk,
}

impl FileDescriptor {
    pub fn new(filesystem: FileSystem, disk: Disk) -> Result<FileDescriptor, ()> {
        let mut file_descriptors = FILE_DESCRIPTORS.lock();
        let new = FileDescriptor { filesystem, disk, };
        file_descriptors.push(new.clone());
        Ok((new))
    }
}

lazy_static! {
    pub static ref FILE_DESCRIPTORS: Mutex<Vec<FileDescriptor>> = Mutex::new(Vec::new());
}

pub fn fs_init() {
    fs_static_load(); 
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

#[derive(Debug)]
pub enum FOpenError {
    ParsePathError,
    NoDiskError,
    NoFilesystemError,
    InvalidFileModeError,
    OpenFileError,
}

pub fn fopen(filename: String, mode: String) -> Result<FileDescriptor, FOpenError> {
    let root_path = path_parser::init_path(filename);
    match root_path {
        Ok(root_path) => {
            let disk = get_disk(root_path.drive_number);
            if disk.is_none() { return Err(FOpenError::NoDiskError) }
            let disk = disk.unwrap();
            let disk = disk.lock();
            if disk.filesystem.is_none() { return Err(FOpenError::NoFilesystemError) }
            let filesystem = disk.filesystem.unwrap();
            let mode = get_file_mode_from_string(mode);
            if matches!(mode, FileMode::INVALID) { return Err(FOpenError::InvalidFileModeError) }
            let first = root_path.first;
            let descriptor_private_data = (filesystem.open)(&disk, &first, &mode);
            if descriptor_private_data.is_err() { return Err(FOpenError::OpenFileError) }
            let file_descriptor = FileDescriptor::new(filesystem, disk.clone());
            if file_descriptor.is_err() { return Err(FOpenError::OpenFileError) }
            Ok(file_descriptor.unwrap())
        },
        Err(_) => Err(FOpenError::ParsePathError),
    }
}

fn get_file_mode_from_string(mode: String) -> FileMode {
    match mode.as_str() {
        "r" => FileMode::READ,
        "w" => FileMode::WRITE,
        "a" => FileMode::APPEND,
        _ => FileMode::INVALID,
    }
}