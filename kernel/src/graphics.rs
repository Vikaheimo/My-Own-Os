use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use font8x8::UnicodeFonts;

const SCREEN_CHARACTER_WIDTH: usize = 8;

pub struct FramebufferWriter {
    framebuffer: FrameBuffer,
    info: FrameBufferInfo,
}

impl FramebufferWriter {
    pub fn new(value: FrameBuffer) -> Self {
        let pixel_format = value.info().pixel_format;
        match pixel_format {
            PixelFormat::Rgb => log::info!("Using rgb pixel format"),
            PixelFormat::Bgr => log::info!("Using bgr pixel format"),
            PixelFormat::U8 => log::info!("Using u8 pixel format"),
            format => log::error!("Unknown pixel format: {format:?}"),
        };
        let info = value.info();
        Self {
            framebuffer: value,
            info,
        }
    }

    pub fn draw_string(&mut self, string: &str, position: Point, color: Color) {
        for (count, c) in string.char_indices() {
            self.draw_char(
                c,
                Point {
                    x: position.x + count * SCREEN_CHARACTER_WIDTH,
                    y: position.y,
                },
                color,
            );
        }
    }

    pub fn draw_char(&mut self, char: char, position: Point, color: Color) {
        if let Some(glyph) = font8x8::BASIC_FONTS.get(char) {
            for (y, row) in glyph.into_iter().enumerate() {
                for x in 0..8 {
                    if !n_th_bit_is_set(row, x) {
                        continue;
                    }

                    let point = Point {
                        x: position.x + x as usize,
                        y: position.y + y,
                    };

                    self.set_pixel(point, color);
                }
            }
        }
    }

    pub fn clear_screen(&mut self, color: Color) {
        let format = self.info.pixel_format;
        for pixel in self.get_pixels_mut() {
            Self::draw_pixel(pixel, format, color);
        }
    }

    pub fn set_pixel(&mut self, point: Point, color: Color) {
        let is_off_screen = point.x >= self.info.width || point.y >= self.info.height;
        if is_off_screen {
            return;
        }

        let offset = point.y * self.info.stride + point.x;
        let format = self.info.pixel_format;
        let pixel = self
            .get_pixels_mut()
            .nth(offset)
            .expect("Pixel should be on screen!");

        Self::draw_pixel(pixel, format, color)
    }

    fn get_pixels_mut(&mut self) -> alloc::slice::ChunksExactMut<'_, u8> {
        self.framebuffer
            .buffer_mut()
            .chunks_exact_mut(self.info.bytes_per_pixel)
    }

    fn draw_pixel(pixel: &mut [u8], format: PixelFormat, color: Color) {
        match format {
            PixelFormat::Rgb => {
                pixel[0] = color.red;
                pixel[1] = color.green;
                pixel[2] = color.blue;
            }
            PixelFormat::Bgr => {
                pixel[0] = color.blue;
                pixel[1] = color.green;
                pixel[2] = color.red;
            }
            PixelFormat::U8 => pixel[0] = color.red,
            _format => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[inline]
fn n_th_bit_is_set(num: u8, n: u8) -> bool {
    (num >> n) & 1 == 1
}
