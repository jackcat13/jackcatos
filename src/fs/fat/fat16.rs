
use alloc::{boxed::Box, string::String, vec::{self, Vec}}; 

use crate::{disk::{disk::Disk, streamer::DiskStreamer}, fs::{file::{FileMode, FileSystem}, path_parser::{PathPart, PATH_MAX_SIZE}}, print, println};

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

#[derive(Debug, Clone, Copy)]
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
    pub filename: [u8; 8],
    pub extension: [u8; 3],
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
#[repr(C)]
pub struct FatDirectory {
    pub items: Vec<FatDirectoryItem>,
    total: u32,
    sector_pos: u32,
    end_sector_pos: u32,
} 

#[derive(Debug, Clone)]
#[repr(C)]
pub struct FatItem {
    item: Option<FatDirectoryItem>,
    directory: Option<FatDirectory>,
    item_type: FatItemType,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct FatFileDescriptor {
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
    let total_items = fat16_get_total_items_for_directory(disk.clone(), fat_private.directory_stream.clone(), root_dir_sector_position as u32);
    if total_items.is_none() { return None; }
    let position = fat16_sector_to_absolute(disk.clone(), root_dir_sector_position) as u32;
    fat_private.directory_stream.seek(position);
    let dir = fat_private.directory_stream.read((root_dir_size -1) as u16);
    if dir.is_none() { return None; }
    let dir = dir.unwrap();
    let dir = unsafe { dir.align_to::<u8>().1 };
    let fat_directory = Some(FatDirectory {
        items: to_fat_directory_items(dir, total_items.unwrap()),
        total: total_items.unwrap() as u32,
        sector_pos: root_dir_sector_position,
        end_sector_pos: (root_dir_sector_position + (root_dir_size / disk.sector_size as u32)),
    });
    // println!("Root directory: {:x?}", fat_directory);
    fat_directory
}

fn fat16_sector_to_absolute(disk: Disk, sector: u32) -> u32 {
    sector * disk.sector_size as u32
}

fn fat16_get_total_items_for_directory(disk: Disk, mut stream: Box<DiskStreamer>, directory_start_sector: u32) -> Option<u16> {
    let mut i = 0;
    let directory_start_position = directory_start_sector * disk.sector_size as u32;
    // println!("Directory start position: {}", directory_start_position);
    stream.seek(directory_start_position);
    // println!("stream pos : {}", stream.pos);
    loop {
        let item = stream.read(size_of::<FatDirectoryItem>() as u16);
        if item.is_none() { continue; }
        let item = item.unwrap();
        let item = unsafe { item.align_to::<u8>().1 };
        let item = to_fat_directory_item(item);
        let filename = item.filename;
        let filename = filename.iter().map(|x| *x as char).collect::<String>();
        let extension = item.extension;
        let extension = extension.iter().map(|x| *x as char).collect::<String>();
        println!("Filename: {:x?}", filename);
        println!("Extension: {:x?}", extension);
        // println!("stream pos : {}", stream.pos);
        
        if item.filename[0] as u8 == 0x00 { break; }
        if item.filename[0] as u8 == 0xE5 { continue; }
        i += 1;
    }
    println!("Total items: {}", i);
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

fn to_fat_directory_items(dir: &[u8], total: u16) -> Vec<FatDirectoryItem> {
    let mut res = alloc::vec![];
    for i in 0..total {
        let item = dir[i as usize * size_of::<FatDirectoryItem>()..(i as usize + 1) * size_of::<FatDirectoryItem>()].to_vec();
        let item = to_fat_directory_item(&item);
        res.push(item);
    }
    res
}

fn to_fat_directory_item(dir: &[u8]) -> FatDirectoryItem {
    let dir = dir.as_ptr();
    let fat_directory_item = unsafe { *(dir as *const FatDirectoryItem) };
    fat_directory_item
}

fn fat16_open(disk: &Disk, path: &PathPart, mode: &FileMode) -> Result<(), ()> {
    if !matches!(mode, FileMode::READ) { return Err(()); }
    // println!("get root directory entry");
    let item = fat16_get_directory_entry(disk, path);
    if item.is_err() { return Err(()); }
    let item = item.unwrap();
    if item.is_none() { return Err(()); }
    let item = item.unwrap();
    // println!("root directory entry type: {:x?}", item.item_type);
    let fat_file_descriptor = FatFileDescriptor {
        position: 0,
        item,
    };
    // print!("return fat file descriptor {:x?}", fat_file_descriptor);
    Ok(())
}

fn fat16_get_directory_entry(disk: &Disk, path: &PathPart) -> Result<Option<FatItem>, ()> {
    let fat_private = disk.fat_private.clone();
    if fat_private.is_none() { return Err(()); }
    let fat_private = fat_private.unwrap();
    let root_directory = fat_private.root_directory;
    if root_directory.is_none() { return Err(()); }
    let root_directory = root_directory.unwrap();
    let part = path.part.clone();
    let root_item = fat16_find_item_in_directory(disk, &root_directory, part);
    if root_item.is_none() { return Err(()); }
    let root_item = root_item.unwrap();
    let mut next_part = &path.next;
    let mut current_item = root_item;
    while next_part.is_some() {
        if !matches!(current_item.item_type, FatItemType::Directory) {
            return Ok(None);
        }
        let directory = current_item.directory.clone();
        let tmp_item = fat16_find_item_in_directory(disk, &(directory.unwrap()), next_part.clone().unwrap().part);
        if tmp_item.is_some() { current_item = tmp_item.unwrap() } 
        next_part = &(next_part.as_ref().unwrap().next);
    }
    Ok(Some(current_item))
}

fn fat16_find_item_in_directory(disk: &Disk, directory: &FatDirectory, name: String) -> Option<FatItem> {
    let mut f_item = Err(());
    for i in 0..directory.total {
        let item = directory.items[i as usize];
        let tmp_filename = fat16_get_full_relative_filename(&item);
        println!("search name compared with : {} / {}", name, tmp_filename);
        if tmp_filename == name {
            println!("found item in directory : {:x?} in {:x?}", tmp_filename, directory.sector_pos);
            f_item = fat16_new_fat_item_for_directory_item(disk, &item);
        }
    }
    if f_item.is_err() { return None; }
    let f_item = f_item.unwrap();
    Some(f_item)
}

fn fat16_get_full_relative_filename(item: &FatDirectoryItem) -> String {
    let filename = item.filename;
    let extension = item.extension;
    let mut res = fat16_to_proper_string(filename.to_vec());
    if extension[0] != 0x00 && extension[0] != 0x20 {
        res.push('.');
        res += &(fat16_to_proper_string(extension.to_vec()));
    }
    res
}

fn fat16_to_proper_string(path: Vec<u8>) -> String {
    let mut res = String::new();
    for c in path.iter() {
        let c = *c;
        if c == 0x00 && c == 0x20 { 
            if c == 0x20 { res.push(0x20u8 as char) }
            break; 
        }
        res.push(c as char);
    }
    String::from(res.to_lowercase().trim())
}

fn fat16_new_fat_item_for_directory_item(disk: &Disk, item: &FatDirectoryItem) -> Result<FatItem, ()> {
    let mut f_item = FatItem {
        item: None,
        directory: None,
        item_type: FatItemType::File,
    };
    if item.attributes & FAT_FILE_SUBDIRECTORY != 0 {
        let directory = fat16_load_fat_directory(disk, item);
        if directory.is_err() { return Err(()); }
        let directory = directory.unwrap();
        f_item.directory = Some(directory);
        f_item.item_type = FatItemType::Directory;
    }
    
    f_item.item = Some(item.clone());
    Ok(f_item)
}

fn fat16_load_fat_directory(disk: &Disk, item: &FatDirectoryItem) -> Result<FatDirectory, ()> {
    let fat_private = disk.fat_private.clone();
    if fat_private.is_none() { return Err(()); }
    let fat_private = fat_private.unwrap();
    if item.attributes & FAT_FILE_SUBDIRECTORY == 0 { return Err(()); }
    let cluster = fat16_get_first_cluster(item);
    let cluster_sector = fat16_cluster_to_sector(&fat_private, cluster);
    if cluster_sector.is_err() { return Err(()); }
    let cluster_sector = cluster_sector.unwrap();
    let total_items = fat16_get_total_items_for_directory(disk.clone(), fat_private.directory_stream, cluster_sector);
    if total_items.is_none() { return Err(()); }
    let total_items = total_items.unwrap();
    let directory_size = total_items + size_of::<FatDirectoryItem>() as u16;
    let item = fat16_read_internal(disk, cluster, 0x00, directory_size);
    if item.is_err() { return Err(()); }
    let item = item.unwrap();
    let item = to_fat_directory_items(unsafe { item.align_to::<u8>().1 }, total_items);
    let directory = FatDirectory {
        items: item,
        total: total_items as u32,
        sector_pos: cluster_sector,
        end_sector_pos: 0,
    };
    Ok(directory)
}

fn fat16_get_first_cluster(item: &FatDirectoryItem) -> u16 {
    (item.high_16_bits_first_cluster) | item.low_16_bits_first_cluster
}

fn fat16_cluster_to_sector(fat_private: &FatPrivate, cluster: u16) -> Result<u32, ()> {
    let root_directory = fat_private.root_directory.clone();
    if root_directory.is_none() { return Err(()); }
    let root_directory = root_directory.unwrap();
    let header = fat_private.header.primary_header;
    Ok(root_directory.end_sector_pos + (cluster as u32 - 2) * header.sectors_per_cluster as u32)
}

fn fat16_read_internal(disk: &Disk, starting_cluster: u16, offset: u16, total: u16) -> Result<Vec<u16>, ()> {
    let fat_private = disk.fat_private.clone();
    if fat_private.is_none() { return Err(()); }
    let fat_private = fat_private.unwrap();
    let stream = fat_private.cluster_read_stream;
    let res = fat16_read_internal_from_stream(disk, &stream, starting_cluster, offset, total);
    if res.is_err() { return Err(()); }
    Ok(res.unwrap())
}

fn fat16_read_internal_from_stream(disk: &Disk, stream: &DiskStreamer, cluster: u16, offset: u16, mut total: u16) -> Result<Vec<u16>, ()> {
    let fat_private = disk.fat_private.clone();
    if fat_private.is_none() { return Err(()); }
    let fat_private = fat_private.unwrap();
    let size_of_cluster_bytes = fat_private.header.primary_header.sectors_per_cluster as u16 * fat_private.header.primary_header.bytes_per_sector;
    let cluster_to_use = fat16_get_cluster_for_offset(disk, cluster, offset);
    if cluster_to_use.is_err() { return Err(()); }
    let cluster_to_use = cluster_to_use.unwrap();
    let offset_from_cluster = offset % size_of_cluster_bytes;
    let starting_sector = fat16_cluster_to_sector(&fat_private, cluster_to_use);
    if starting_sector.is_err() { return Err(()); }
    let starting_sector = starting_sector.unwrap();
    let starting_pos = (starting_sector * disk.sector_size as u32) * offset_from_cluster as u32;
    let total_to_read = if total > size_of_cluster_bytes { size_of_cluster_bytes } else { total };
    let mut stream = stream.clone();
    stream.seek(starting_pos);
    let res = stream.read(total_to_read);
    if res.is_none() { return Err(()); }
    let mut res = res.unwrap();
    total -= total_to_read;
    if (total > 0) {
        let res2 = fat16_read_internal_from_stream(disk, &stream, cluster, offset + total_to_read, total);
        if res2.is_err() { return Err(()); }
        let mut res2 = res2.unwrap();
        res.append(&mut res2);
    }
    Ok(res)
}

fn fat16_get_cluster_for_offset(disk: &Disk, cluster: u16, offset: u16) -> Result<u16, ()> {
    let fat_private = disk.fat_private.clone();
    if fat_private.is_none() { return Err(()); }
    let fat_private = fat_private.unwrap();
    let size_of_cluster_bytes = fat_private.header.primary_header.sectors_per_cluster as u16 * disk.sector_size;
    let mut cluster_to_use = cluster;
    let cluster_ahead = offset / size_of_cluster_bytes;
    for _ in 0..cluster_ahead {
        let entry = fat16_get_fat_entry(disk, cluster_to_use);
        if entry.is_err() { return Err(()); }
        let entry = entry.unwrap();
        if entry == 0xFF8 || entry == 0xFFF { return Err(()); }
        if entry == FAT_16_BAD_SECTOR { return Err(()); }
        if entry == 0xFF0 || entry == 0xFF6 { return Err(()); }
        if entry == 0x00 { return Err(()); }
        cluster_to_use = entry;
    }
    Ok(cluster_to_use)
}

fn fat16_get_fat_entry(disk: &Disk, cluster_to_use: u16) -> Result<u16, ()> {
    let fat_private = disk.fat_private.clone();
    if fat_private.is_none() { return Err(()); }
    let fat_private = fat_private.unwrap();
    let mut stream = fat_private.clone().fat_read_stream;
    let fat_table_position = fat16_get_first_fat_sector(&fat_private) * disk.sector_size;
    stream.seek((fat_table_position * (cluster_to_use * FAT_16_FAT_ENTRY_SIZE as u16)) as u32);
    let res = stream.read(16);
    if res.is_none() { return Err(()); }
    Ok(*(res.unwrap().get(0).unwrap()))
}

fn fat16_get_first_fat_sector(fat_private: &FatPrivate) -> u16 {
    fat_private.header.primary_header.reserved_sectors
}