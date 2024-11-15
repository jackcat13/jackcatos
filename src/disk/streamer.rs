
use alloc::{boxed::Box, vec};
use crate::disk::disk::SECTOR_SIZE;

use super::disk::{disk_read_block, Disk};

#[derive(Debug, Clone)]
pub struct DiskStreamer {
    pub pos: u32,
    disk: Disk,
}

impl DiskStreamer {
    pub fn new(disk: Disk) -> Box<DiskStreamer> {
        Box::new(DiskStreamer {
            pos: 0,
            disk,
        })
    }
    
    pub fn read(&mut self, total: u32) -> Option<vec::Vec<u16>> {
        let sector = self.pos / SECTOR_SIZE as u32;
        
        let data = disk_read_block(self.disk.clone(), sector, 1);
        let total_to_read = if total > SECTOR_SIZE { SECTOR_SIZE } else { total };
        match data {
            None => None,
            Some(data) => {
                let rel_pos = self.pos % SECTOR_SIZE as u32;
                let mut res = data[rel_pos as usize..(rel_pos + total_to_read as u32) as usize].to_vec();
                self.pos += total_to_read as u32;
                if total > SECTOR_SIZE {
                    let rest = self.read(total - SECTOR_SIZE);
                    if let Some(mut rest) = rest {
                        res.append(&mut rest);
                    }
                }
                Some(res)
            }
        }
    }
    
    pub fn seek(&mut self, pos: u32) {
        self.pos = pos;
    }
}