use alloc::{boxed::Box, vec};
use x86_64::structures::port::{PortRead, PortWrite};

use crate::println;

pub const SECTOR_SIZE: u16 = 512;

#[derive(Debug, Clone, Copy)]
pub enum DiskType {
    OsDisk
}

#[derive(Debug, Clone, Copy)]
pub struct Disk {
    pub type_: DiskType,
    pub sector_size: u16,
    pub disk_number: u8,
}

fn disk_search_and_init() -> Disk {
    let disk = Disk {
        type_: DiskType::OsDisk,
        sector_size: SECTOR_SIZE,
        disk_number: 0,
    };

    disk
}

pub fn get_disk(index: u8) -> Option<Box<Disk>> {
    if index != 0 {
        return None; // Only one disk is supported so far
    } 
    Some(Box::new(disk_search_and_init()))
}

pub fn disk_read_block(disk: Box<Disk>, lba: u32, total: u8) -> Option<vec::Vec<u16>> {
    if disk.disk_number != 0 {
        return None; // Only one disk is supported so far
    }
    Some(disk_read_sector(lba, total))
}

fn disk_read_sector(lba: u32, total: u8) -> vec::Vec<u16>{ 
    unsafe { PortWrite::write_to_port(0x1F6, (lba >> 24) as u8 | 0xE0) };
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
            println!("{:x?}", (c & mask));
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
