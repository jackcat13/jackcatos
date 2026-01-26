use core::mem::size_of;
use crate::color::Color;
use crate::vbe::get_vbe;

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_middle: u16,
    offset_high: u32,
    zero: u32,
}

/// Pointer to load into IDTR register
#[repr(C, packed)]
pub struct IdtPtr {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::new(); 256];

impl IdtEntry {
    pub const fn new() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_middle: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u64) {
        self.offset_low = handler as u16;
        // CRITICAL: This must match the CODE_SEG_64 in your gdt.asm (0x18)
        self.selector = 0x18;
        self.ist = 0;
        // 0x8E = Present, Ring 0, Interrupt Gate
        self.type_attr = 0x8E;
        self.offset_middle = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
    }
}

pub fn init_idt() {
    unsafe {
        // Set Breakpoint Handler (Vector 3)
        // This is useful for testing interrupts without crashing
        IDT[3].set_handler(breakpoint_handler as *const () as u64);

        // Load the IDT using the 'lidt' assembly instruction
        let ptr = IdtPtr {
            limit: (size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };

        core::arch::asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
    }
}

// The Breakpoint Exception Handler
// We use the special calling convention "x86-interrupt"
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    // Since we are in graphics mode, we can't easily print text unless we pass the
    // VbeInfo around. For now, let's just modify the screen directly to prove it works.

    // Read VBE info from 0x5000 (just like in main)
    // In a real OS, you'd have a static global Writer
    let vbe_info = get_vbe();

    // Draw a White square at (150, 150) to signal "Interrupt Handled!"
    vbe_info.draw_square(200, 200, 100, Color{red: 0xFF, green: 0xFF, blue: 0xFF});
}