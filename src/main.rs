#![feature(custom_test_frameworks)]
#![test_runner(jackcatos::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use jackcatos::{hlt_loop, println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Welcome to the JackCatOs :)");
    
    jackcatos::init();
    
    #[cfg(test)]
    test_main();

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
