#![feature(custom_test_frameworks)]
#![test_runner(jackcatos::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use jackcatos::{
    allocator, disk, fs::path_parser::init_path, hlt_loop, memory::{self, BootInfoFrameAllocator}, println, task::{executor::Executor, keyboard, Task}
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
    
    let path = "1:/dzaopdkzaopddzaopdkzaopddzaopdkzaopddzaopdkzaopd/koapdz".to_string();
    let p = init_path(path);
    println!("{:?}", p);
    
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
