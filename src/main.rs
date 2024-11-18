#![feature(custom_test_frameworks)]
#![test_runner(jackcatos::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use jackcatos::{
    allocator, disk::{self}, fs::file::{fopen, fread, fseek, FileSeekMode::SEEK_SET}, hlt_loop, memory::{self, BootInfoFrameAllocator}, println, task::{executor::Executor, keyboard, Task}
};
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Welcome to the JackCatOs :)");

    jackcatos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap init failed");
    // let disk = disk::disk::get_disk(0).unwrap();
    let disk2 = disk::disk::get_disk(1).unwrap();
    let disk2 = disk2.clone();
    let disk2 = disk2.lock();
    let disk2 = disk2.clone();
    // let disk_stream = DiskStreamer::new(disk2.clone());
    
    // println!("Disk content : {:x?}", disk2);
    let fat_private = *disk2.fat_private.unwrap();
    let header = fat_private.header;
    let extended_header = header.extended_header;
    let volume_id_string = extended_header.volume_id_string;
    // println!("volume label : {:x?}", volume_id_string);
    let mut fd = fopen("1:/test.txt".to_string(), "r".to_string()).unwrap();
    println!("Fopen text : {:x?}", fd.private.clone().iter().map(|x| x.clone() as char).collect::<String>());
    fseek(&mut fd, 2, SEEK_SET);
    let read = fread(fd.private, 5, 1, fd.index);
    let read = read.unwrap();
    println!("Fread test.txt : {:x?}", read.iter().map(|x| x.clone() as char).collect::<String>());
    
    #[cfg(test)]
    test_main();
    
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    hlt_loop();
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use jackcatos::hlt_loop;

    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    jackcatos::test_panic_handler(info)
}
