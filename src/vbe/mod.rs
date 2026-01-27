use crate::color::Color;
use crate::VBE_MODE_INFO_ADDRESS;

#[repr(packed)]
pub struct VbeModeInfo {
    attributes: u16,
    window_a: u8,
    window_b: u8,
    granularity: u16,
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32,
    pitch: u16, // Bytes per scanline
    width: u16, // Resolution X
    height: u16, // Resolution Y
    w_char: u8,
    y_char: u8,
    planes: u8,
    bpp: u8, // Bits per pixel (e.g., 24 or 32)
    banks: u8,
    memory_model: u8,
    bank_size: u8,
    image_pages: u8,
    reserved0: u8,
    red_mask: u8,
    red_position: u8,
    green_mask: u8,
    green_position: u8,
    blue_mask: u8,
    blue_position: u8,
    reserved_mask: u8,
    reserved_position: u8,
    direct_color_attributes: u8,
    framebuffer: u32, // <--- Physical Address of the video memory!
    // ... rest omitted for brevity
}

impl VbeModeInfo {
    pub fn clear_background(&self, color: Color) {
        let width = self.width as isize;
        let height = self.height as isize;
        for y in 0..height {
            for x in 0..width {
                self.draw_pixel(x, y, color);
            }
        }
    }

    /// Draw a square at (x, y) with size square_size
    pub fn draw_square(&self, x: isize, y: isize, square_size: isize, color: Color){
        for y in y..(y + square_size) {
            for x in x..(x + square_size) {
                self.draw_pixel(x, y, color);
            }
        }
    }

    pub fn draw_pixel(&self, x: isize, y: isize, color: Color) {
        let framebuffer = self.framebuffer as *mut u8;
        let pitch = self.pitch as isize;
        let bpp = self.bpp as isize;
        let bytes_per_pixel = bpp / 8;
        let offset = y * pitch + x * bytes_per_pixel;
        unsafe {
            *framebuffer.offset(offset) = color.blue;
            *framebuffer.offset(offset + 1) = color.green;
            *framebuffer.offset(offset + 2) = color.red;
        }
    }

    pub fn draw_text(&self, x: isize, y: isize, text: &str, color: Color) {
        let mut x_offset = x;
        for char in text.chars() {
            match char {
                'w' | 'W' => self.draw_pixel(x_offset, y, color),
                _ => { self.draw_square(x_offset, y, 16, color) }
            }
            x_offset += 40;
        }
    }
}

pub fn get_vbe<'a>() -> &'a VbeModeInfo {
    unsafe { &*(VBE_MODE_INFO_ADDRESS as *const VbeModeInfo) }
}