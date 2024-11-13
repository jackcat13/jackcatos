
use alloc::boxed::Box;

use crate::{disk::{disk::Disk, streamer::DiskStreamer}, fs::{file::{FileMode, FileSystem}, path_parser::PathPart}, println};

pub const FAT_16_SIGNATURE: u8 = 0x29;
pub const FAT_16_FAT_ENTRY_SIZE: u8 = 0x02;
pub const FAT_16_BAD_SECTOR: u16 = 0xFF7;
pub const FAT_16_UNUSED: u8 = 0x00;

pub const FAT_FILE_READ_ONLY: u8 = 0x01;
pub const FAT_FILE_HIDDEN: u8 = 0x02;
pub const FAT_FILE_SYSTEM: u8 = 0x04;
pub const FAT_FILE_VOLUME_LABEL: u8 = 0x08;
pub const FAT_FILE_SUBDIRECTORY: u8 = 0x10;
pub const FAT_FILE_ARCHIVE: u8 = 0x20;
pub const FAT_FILE_DEVICE: u8 = 0x40;
pub const FAT_FILE_RESERVED: u8 = 0x80;

pub enum FatItemType {
    Directory, File
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FatHeaderExtended {
    pub drive_number: u8,
    pub win_nt_bit: u8,
    pub signature: u8,
    pub volume_id: u32,
    pub volume_id_string: [u8; 11],
    pub system_id_string: [u8; 8], 
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FatHeader {
    short_jmp_instruction: [u8; 3],
    oem_identifier: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_copies: u8,
    root_dir_entries: u16,
    number_of_sectors: u16,
    media_type : u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    number_of_heads: u16,
    hidden_sectors: u32,
    sectors_big: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FatH {
    pub primary_header: FatHeader,
    pub extended_header: FatHeaderExtended,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct FatDirectoryItem {
    pub filename: [char; 8],
    pub extension: [char; 3],
    pub attributes: u8,
    pub reserved: u8,
    pub creation_time_tenths_of_a_sec: u8,
    pub creation_time: u16,
    pub creation_date: u16,
    pub last_access: u16,
    pub high_16_bits_first_cluster: u16,
    pub last_modification_time: u16,
    pub last_modification_date: u16,
    pub low_16_bits_first_cluster: u16,
    pub file_size: u32,
}

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct FatDirectory {
    pub item: FatDirectoryItem,
    total: u32,
    sector_pos: u32,
    end_sector_pos: u32,
} 

#[repr(C, packed)]
pub struct FatItem {
    item: Option<FatDirectoryItem>,
    directory: Option<FatDirectory>,
    item_type: FatItemType,
}

#[repr(C, packed)]
pub struct FatItemDescriptor {
    item: FatItem,
    position: u32,
}

#[derive(Debug, Clone)]
pub struct FatPrivate {
    pub header: FatH,
    pub root_directory: Option<FatDirectory>,
    
    // Used to stream data clusters
    cluster_read_stream: Box<DiskStreamer>,
    // Used to stream the file alocation table
    fat_read_stream: Box<DiskStreamer>,
    // Used in situation where we stream the directory
    directory_stream: Box<DiskStreamer>,
}

pub fn fat16_init() -> FileSystem {
    FileSystem {
        resolve: fat16_resolve,
        open: fat16_open,
        
        name: ['f', 'a', 't', '1', '6', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    }
}

#[derive(Debug)]
pub enum ResolveError {
    InvalidSignature,
    NoExtendedHeader,
    FailedToReadHeader,
    FailedToGetRootDirectory
}

fn fat16_resolve(disk: Disk) -> Result<Box<FatPrivate>, ResolveError> {
    let mut disk_stream = DiskStreamer::new(disk.clone());
    let fat_h = disk_stream.read(size_of::<FatH>() as u16);
    if fat_h == None { return Err(ResolveError::FailedToReadHeader); }
    let fat_h = fat_h.unwrap();
    let fat_h = unsafe { fat_h.align_to::<u8>().1 };
    let header: FatH = to_fat_h(fat_h);
    // print_header_details(&header);
    if header.extended_header.signature != 0x29 { return Err(ResolveError::InvalidSignature); }
    println!("Signature: {:x?}", header.extended_header.signature);
    let mut fat_private = Box::new(FatPrivate {
        cluster_read_stream: DiskStreamer::new(disk.clone()),
        fat_read_stream: DiskStreamer::new(disk.clone()),
        directory_stream: DiskStreamer::new(disk.clone()),
        header,
        root_directory: None,
    });
    let root_directory = match fat16_get_root_directory(disk, &header.primary_header, fat_private.clone()) {
        None => return Err(ResolveError::FailedToGetRootDirectory),
        Some(d) => d,
    };
    fat_private.header = header;
    fat_private.root_directory = Some(root_directory);
    Ok(fat_private)
}

fn print_header_details(header: &FatH) {
    let bytes_per_sector = header.primary_header.bytes_per_sector;
    let sectors_per_cluster = header.primary_header.sectors_per_cluster;
    let reserved_sectors = header.primary_header.reserved_sectors;
    let fat_copies = header.primary_header.fat_copies;
    let root_dir_entries = header.primary_header.root_dir_entries;
    let number_of_sectors = header.primary_header.number_of_sectors;
    let media_type = header.primary_header.media_type;
    let sectors_per_fat = header.primary_header.sectors_per_fat;
    let sectors_per_track = header.primary_header.sectors_per_track;
    let number_of_heads = header.primary_header.number_of_heads;
    let hidden_sectors = header.primary_header.hidden_sectors;
    let sectors_big = header.primary_header.sectors_big;
    println!("Bytes per sector: {}", bytes_per_sector);
    println!("Sectors per cluster: {}", sectors_per_cluster);
    println!("Reserved sectors: {}", reserved_sectors);
    println!("FAT copies: {}", fat_copies);
    println!("Root directory entries: {}", root_dir_entries);
    println!("Number of sectors: {}", number_of_sectors);
    println!("Media type: {}", media_type);
    println!("Sectors per FAT: {}", sectors_per_fat);
    println!("Sectors per track: {}", sectors_per_track);
    println!("Number of heads: {}", number_of_heads);
    println!("Hidden sectors: {}", hidden_sectors);
    println!("Sectors big: {}", sectors_big);
    
    let drive_number = header.extended_header.drive_number;
    let windows_nt_flags = header.extended_header.win_nt_bit;
    let signature = header.extended_header.signature;
    let volume_id = header.extended_header.volume_id;
    let volume_id_string = header.extended_header.volume_id_string;
    let system_id_string = header.extended_header.system_id_string;
    println!("Drive number: {}", drive_number);
    println!("Windows NT flags: {}", windows_nt_flags);
    println!("Signature: {}", signature);
    println!("Volume ID: {}", volume_id);
    println!("System ID string: {:?}", system_id_string);
    
}

fn to_fat_h(fat_h: &[u8]) -> FatH {
    let mut ptr = fat_h.as_ptr();
    let primary_header = unsafe { *(ptr as *const FatHeader) };
    ptr = unsafe { ptr.add(size_of::<FatHeader>()) };
    let extended_header = unsafe { *(ptr as *const FatHeaderExtended) };
    FatH { primary_header, extended_header, }
}

fn fat16_get_root_directory(disk: Disk, primary_header: &FatHeader, mut fat_private: Box<FatPrivate>) -> Option<FatDirectory> {
    let root_dir_sector_position = (primary_header.fat_copies as u32 * primary_header.sectors_per_fat as u32) + primary_header.reserved_sectors as u32;
    let root_dir_size = primary_header.root_dir_entries as u32 * size_of::<FatDirectoryItem>() as u32;
    let mut total_sectors = root_dir_size / disk.sector_size as u32;
    if (root_dir_size % disk.sector_size as u32) != 0 { total_sectors += 1; }
    let total_items = fat16_get_total_items_for_directory(disk.clone(), fat_private.clone(), root_dir_sector_position as u32);
    if total_items.is_none() { return None; }
    // println!("root dir sector position: {}", root_dir_sector_position);
    // println!("root dir size: {}", root_dir_size);
    // println!("Total sectors: {}", total_sectors);
    // println!("Total items: {}", total_items.unwrap());
    fat_private.directory_stream.seek(fat16_sector_to_absolute(disk.clone(), root_dir_sector_position) as u32);
    let dir = fat_private.directory_stream.read(root_dir_size as u16);
    if dir.is_none() { return None; }
    let dir = dir.unwrap();
    let dir = unsafe { dir.align_to::<u8>().1 };
    let fat_directory = Some(FatDirectory {
        item: to_fat_directory_item(dir),
        total: total_items.unwrap(),
        sector_pos: root_dir_sector_position,
        end_sector_pos: (root_dir_sector_position + (root_dir_size / disk.sector_size as u32)),
    });
    // println!("Root directory: {:x?}", fat_directory);
    fat_directory
}

fn fat16_sector_to_absolute(disk: Disk, sector: u32) -> u32 {
    sector * disk.sector_size as u32
}

fn fat16_get_total_items_for_directory(disk: Disk, fat_private: Box<FatPrivate>, directory_start_sector: u32) -> Option<u32> {
    let mut i = 0;
    let directory_start_position = directory_start_sector * disk.sector_size as u32;
    let mut stream = fat_private.directory_stream;
    stream.seek(directory_start_position);
    let item = stream.read(size_of::<FatDirectoryItem>() as u16);
    if item.is_none() { return None; }
    let item = item.unwrap();
    let item = unsafe { item.align_to::<u8>().1 };
    let item = to_fat_directory_item(item);
    // print_item_details(&item);
    for c in item.filename {
        let c = c as u8;
        if c == 0x00 { break; } // We have finished
        if c == 0xE5 { continue; } // Is the item unused
        i += 1;
    }
    println!("Number of items for directory: {}", i);
    Some(i)
}

fn print_item_details(item: &FatDirectoryItem) {
    let filename = item.filename;
    let extension = item.extension;
    let attributes = item.attributes;
    let reserved = item.reserved;
    let creation_time_tenths_of_a_sec = item.creation_time_tenths_of_a_sec;
    let creation_time = item.creation_time;
    let creation_date = item.creation_date;
    let last_access = item.last_access;
    let high_16_bits_first_cluster = item.high_16_bits_first_cluster;
    let last_modification_time = item.last_modification_time;
    let last_modification_date = item.last_modification_date;
    let low_16_bits_first_cluster = item.low_16_bits_first_cluster;
    let file_size = item.file_size;
    println!("Filename: {:?}", filename);
    println!("Extension: {:?}", extension);
    println!("Attributes: {:?}", attributes);
    println!("Reserved: {:?}", reserved);
    println!("Creation time tenths of a second: {:?}", creation_time_tenths_of_a_sec);
    println!("Creation time: {:?}", creation_time);
    println!("Creation date: {:?}", creation_date);
    println!("Last access: {:?}", last_access);
    println!("High 16 bits first cluster: {:?}", high_16_bits_first_cluster);
    println!("Last modification time: {:?}", last_modification_time);
    println!("Last modification date: {:?}", last_modification_date);
    println!("Low 16 bits first cluster: {:?}", low_16_bits_first_cluster);
    println!("File size: {:?}", file_size);
}

fn to_fat_directory_item(dir: &[u8]) -> FatDirectoryItem {
    let dir = dir.as_ptr();
    let fat_directory_item = unsafe { *(dir as *const FatDirectoryItem) };
    fat_directory_item
}

fn fat16_open(disk: Box<Disk>, path: &PathPart, mode: &FileMode) -> Option<u8> {
    Some(0)
}