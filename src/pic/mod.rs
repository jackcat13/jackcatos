use crate::io::{inb, outb, wait};

// Offsets for the PICs (Master starts at 32, Slave at 40)
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 40;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16    = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16    = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

pub fn init_pic() {
    unsafe {
        // Save masks (we'll ignore them for now and reset them later)
        let _a1 = inb(PIC1_DATA);
        let _a2 = inb(PIC2_DATA);

        // Start initialization sequence
        outb(PIC1_COMMAND, ICW1_INIT);
        wait();
        outb(PIC2_COMMAND, ICW1_INIT);
        wait();

        // Remap offsets
        outb(PIC1_DATA, PIC_1_OFFSET);
        wait();
        outb(PIC2_DATA, PIC_2_OFFSET);
        wait();

        // Tell Master there is a Slave at IRQ2
        outb(PIC1_DATA, 4);
        wait();
        // Tell Slave its cascade identity
        outb(PIC2_DATA, 2);
        wait();

        // Use 8086 mode
        outb(PIC1_DATA, ICW4_8086);
        wait();
        outb(PIC2_DATA, ICW4_8086);
        wait();

        // Unmask interrupts
        // 0 = Enable, 1 = Disable
        // For now, let's enable only the Keyboard (IRQ 1, bit 1)
        // 1111 1101 = 0xFD
        outb(PIC1_DATA, 0b11111101);
        outb(PIC2_DATA, 0b11111111);
    }
}

/// Signal "End of Interrupt" to the PIC so it can send the next one
pub unsafe fn notify_eoi(interrupt_id: u8) {
    if interrupt_id >= PIC_2_OFFSET {
        outb(PIC2_COMMAND, 0x20);
    }
    outb(PIC1_COMMAND, 0x20);
}