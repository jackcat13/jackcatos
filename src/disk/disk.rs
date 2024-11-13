use alloc::{boxed::Box, sync::Arc, vec};
use alloc::rc::Rc;
use spin::Mutex;
use x86_64::structures::port::{PortRead, PortWrite};

use crate::{fs::{fat::fat16::FatPrivate, file::{fs_resolve, FileSystem}}, println};

pub const SECTOR_SIZE: u16 = 512;

#[derive(Debug, Clone, Copy)]
pub enum DiskType {
    OsDisk
}

#[derive(Debug, Clone)]
pub struct Disk {
    pub type_: DiskType,
    pub sector_size: u16,
    pub disk_number: u8,
    pub filesystem: Option<FileSystem>,
    pub fat_private: Option<Box<FatPrivate>>,
}

fn disk_search_and_init(index: u8) -> Arc<Mutex<Disk>> {
    let disk = Arc::new(Mutex::new(Disk {
        type_: DiskType::OsDisk,
        sector_size: SECTOR_SIZE,
        disk_number: index,
        filesystem: None,
        fat_private: None,
    }));
    println!("Start resolving disk");
    fs_resolve(disk.clone()).unwrap();
    println!("End resolving disk");
    disk
}

pub fn get_disk(index: u8) -> Option<Arc<Mutex<Disk>>> {
    if index > 2 {
        return None; // Only one disk are supported so far
    } 
    Some(disk_search_and_init(index))
}

pub fn disk_read_block(disk: Disk, lba: u32, total: u8) -> Option<vec::Vec<u16>> {
    if disk.disk_number >= 2 {
        return None; // Only two disks are supported so far
    }
    if disk.disk_number == 0 {
        let not_slave = 0;
        Some(disk_read_sector(lba, total, not_slave))
    } else {
        let slave = 1;
        Some(disk_read_sector(lba, total, slave))
    }
}

fn disk_read_sector(lba: u32, total: u8, slave_bit: u8) -> vec::Vec<u16>{ 
    unsafe { PortWrite::write_to_port(0x1F6, (slave_bit <<  4) | (lba >> 24) as u8 | 0xE0) };
    unsafe { PortWrite::write_to_port(0x1F2, total) };
    unsafe { PortWrite::write_to_port(0x1F3, (lba & 0xff) as u8) };
    unsafe { PortWrite::write_to_port(0x1F4, (lba >> 8) as u8) };
    unsafe { PortWrite::write_to_port(0x1F5, (lba >> 16) as u8) };
    unsafe { PortWrite::write_to_port(0x1F7, 0x20 as u8) };

    let mut buffer = vec![];
    for _ in 0..total {
        // Wait for the drive to be ready
        let mut c: u8 = unsafe { PortRead::read_from_port(0x1F7) };
        let mask = 1 << 7;
        while (c & mask) == 0x80 {
            c = unsafe { PortRead::read_from_port(0x1F7) };
        }

        // Copy from hard disk to memory
        for _ in 0..SECTOR_SIZE {
            let data = unsafe { PortRead::read_from_port(0x1F0) };
            buffer.push(data);
        }
    }
    buffer
}