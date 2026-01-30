use crate::VBE_MODE_INFO_ADDRESS;
use crate::color::Color;
use crate::vbe::no_font::{NO_FONT_HEIGHT, NO_FONT_WIDTH};

mod no_font;

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
    pitch: u16,  // Bytes per scanline
    width: u16,  // Resolution X
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
        let width = self.width as usize;
        let height = self.height as usize;
        for y in 0..height {
            for x in 0..width {
                self.draw_pixel(x, y, color);
            }
        }
    }

    /// Draw a square at (x, y) with size square_size
    pub fn draw_square(&self, x: usize, y: usize, square_size: usize, color: Color) {
        for y in y..(y + square_size) {
            for x in x..(x + square_size) {
                self.draw_pixel(x, y, color);
            }
        }
    }

    pub fn draw_pixel(&self, x: usize, y: usize, color: Color) {
        let framebuffer = self.framebuffer as *mut u8;
        let pitch = self.pitch as usize;
        let bpp = self.bpp as usize;
        let bytes_per_pixel = bpp / 8;
        let offset = y * pitch + x * bytes_per_pixel;
        unsafe {
            *framebuffer.add(offset) = color.blue;
            *framebuffer.add(offset + 1) = color.green;
            *framebuffer.add(offset + 2) = color.red;
        }
    }

    pub fn draw_text(&self, x: usize, y: usize, text: &str, color: Color) {
        let mut x_offset = x;
        for char in text.chars() {
            match char {
                'a' | 'A' => self.draw_no_font_sprite(x_offset, y, no_font::A, color),
                'b' | 'B' => self.draw_no_font_sprite(x_offset, y, no_font::B, color),
                'c' | 'C' => self.draw_no_font_sprite(x_offset, y, no_font::C, color),
                'd' | 'D' => self.draw_no_font_sprite(x_offset, y, no_font::D, color),
                'e' | 'E' => self.draw_no_font_sprite(x_offset, y, no_font::E, color),
                'f' | 'F' => self.draw_no_font_sprite(x_offset, y, no_font::F, color),
                'g' | 'G' => self.draw_no_font_sprite(x_offset, y, no_font::G, color),
                'h' | 'H' => self.draw_no_font_sprite(x_offset, y, no_font::H, color),
                'i' | 'I' => self.draw_no_font_sprite(x_offset, y, no_font::I, color),
                'j' | 'J' => self.draw_no_font_sprite(x_offset, y, no_font::J, color),
                'k' | 'K' => self.draw_no_font_sprite(x_offset, y, no_font::K, color),
                'l' | 'L' => self.draw_no_font_sprite(x_offset, y, no_font::L, color),
                'm' | 'M' => self.draw_no_font_sprite(x_offset, y, no_font::M, color),
                'n' | 'N' => self.draw_no_font_sprite(x_offset, y, no_font::N, color),
                'o' | 'O' => self.draw_no_font_sprite(x_offset, y, no_font::O, color),
                'p' | 'P' => self.draw_no_font_sprite(x_offset, y, no_font::P, color),
                'q' | 'Q' => self.draw_no_font_sprite(x_offset, y, no_font::Q, color),
                'r' | 'R' => self.draw_no_font_sprite(x_offset, y, no_font::R, color),
                's' | 'S' => self.draw_no_font_sprite(x_offset, y, no_font::S, color),
                't' | 'T' => self.draw_no_font_sprite(x_offset, y, no_font::T, color),
                'u' | 'U' => self.draw_no_font_sprite(x_offset, y, no_font::U, color),
                'v' | 'V' => self.draw_no_font_sprite(x_offset, y, no_font::V, color),
                'w' | 'W' => self.draw_no_font_sprite(x_offset, y, no_font::W, color),
                'x' | 'X' => self.draw_no_font_sprite(x_offset, y, no_font::X, color),
                'y' | 'Y' => self.draw_no_font_sprite(x_offset, y, no_font::Y, color),
                'z' | 'Z' => self.draw_no_font_sprite(x_offset, y, no_font::Z, color),
                ' ' => (),
                _ => self.draw_square(x_offset, y, 10, color),
            }
            x_offset += 13;
        }
    }

    pub fn draw_no_font_sprite(&self, x: usize, y: usize, font_sprite: [[i32; NO_FONT_WIDTH]; NO_FONT_HEIGHT], color: Color) {
        for ny in y..y + NO_FONT_HEIGHT {
            for nx in x..x + NO_FONT_WIDTH {
                if font_sprite[ny - y][nx - x] == 0 {
                    continue;
                }
                self.draw_pixel(nx, ny, color);
            }
        }
    }
}

pub fn get_vbe<'a>() -> &'a VbeModeInfo {
    unsafe { &*(VBE_MODE_INFO_ADDRESS as *const VbeModeInfo) }
}
