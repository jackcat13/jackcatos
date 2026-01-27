use core::arch::asm;

/// Write a byte to a port
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

/// Read a byte from a port
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

/// Wait a very small amount of time (used for synchronizing with slow hardware)
pub unsafe fn wait() {
    outb(0x80, 0);
}