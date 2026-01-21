#![no_std]
#![no_main]

use core::panic::PanicInfo;

// This function is called from assembly
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    // Clear screen (set background to Blue)
    for i in (0..80 * 25 * 2).step_by(2) {
        unsafe {
            *vga_buffer.offset(i as isize) = b' ';      // Space
            *vga_buffer.offset(i as isize + 1) = 0x1F;  // White on Blue
        }
    }

    // Print "Hello from Rust!" at the top
    let msg = b"Hello from Rust!";
    for (i, &byte) in msg.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0x1F;
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}