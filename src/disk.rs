use alloc::vec;
use x86_64::structures::port::{PortRead, PortWrite};

use crate::println;

const BLOCK_SIZE: u16 = 256;

pub fn disk_read_sector(lba: u32, total: u8) -> vec::Vec<u16>{ 
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
        for _ in 0..BLOCK_SIZE {
            let data = unsafe { PortRead::read_from_port(0x1F0) };
            buffer.push(data);
        }
    }
    buffer
}
