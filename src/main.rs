#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use idt::init_idt;
use crate::color::Color;
use crate::vbe::{get_vbe};

mod color;
mod idt;
mod vbe;

const VBE_MODE_INFO_ADDRESS: u16 = 0x5000;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let vbe_info = get_vbe();

    vbe_info.clear_background(Color{ red: 0x00, green: 0x11, blue: 0x33});

    init_idt();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}