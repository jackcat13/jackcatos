#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Multiboot header for GRUB
#[repr(C, align(4))]
struct MultibootHeader {
    magic: u32,
    flags: u32,
    checksum: u32,
}

#[link_section = ".multiboot"]
#[used]
static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader {
    magic: 0x1BADB002,
    flags: 0x0,
    checksum: 0u32.wrapping_sub(0x1BADB002),
};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Write 'OK' to VGA buffer
    let vga_buffer = 0xb8000 as *mut u8;
    unsafe {
        *vga_buffer.offset(0) = b'O';
        *vga_buffer.offset(1) = 0x0f; // White on black
        *vga_buffer.offset(2) = b'K';
        *vga_buffer.offset(3) = 0x0f;
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}