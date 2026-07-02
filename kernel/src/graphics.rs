use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use font8x8::UnicodeFonts;

/// Width of a single rendered character in pixels.
///
/// The `font8x8` crate provides 8×8 bitmap glyphs, so each character
/// occupies 8 horizontal pixels.
const SCREEN_CHARACTER_WIDTH: usize = 8;

/// A simple framebuffer text and pixel renderer.
///
/// `FramebufferWriter` provides basic drawing primitives for:
/// - Clearing the screen
/// - Drawing individual pixels
/// - Rendering characters using an 8×8 bitmap font
/// - Rendering strings horizontally
///
/// It supports multiple pixel formats (`Rgb`, `Bgr`, `U8`) as provided
/// by the bootloader framebuffer API.
pub struct FramebufferWriter {
    /// The underlying framebuffer provided by the bootloader.
    framebuffer: FrameBuffer,

    /// Cached framebuffer metadata for quick access.
    info: FrameBufferInfo,
}

impl FramebufferWriter {
    /// Creates a new `FramebufferWriter` from a bootloader `FrameBuffer`.
    ///
    /// Logs the detected pixel format for debugging purposes.
    ///
    /// # Parameters
    ///
    /// - `value`: The framebuffer provided by the bootloader.
    ///
    /// # Returns
    ///
    /// A new `FramebufferWriter` instance ready for drawing.
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

    /// Draws a UTF‑8 string at the given position.
    ///
    /// Characters are rendered horizontally using the built‑in 8×8 bitmap
    /// font. Each character advances by [`SCREEN_CHARACTER_WIDTH`] pixels.
    ///
    /// # Parameters
    ///
    /// - `string`: The text to render.
    /// - `position`: The top-left starting position in pixels.
    /// - `color`: The color used for rendering the characters.
    ///
    /// # Notes
    ///
    /// Characters not present in the `font8x8::BASIC_FONTS` set
    /// will be skipped.
    pub fn draw_string(&mut self, string: &str, position: Point, color: Color) {
        for (count, c) in string.chars().enumerate() {
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

    /// Draws a single character at the given position.
    ///
    /// The character is rendered using an 8×8 bitmap from the
    /// `font8x8::BASIC_FONTS` collection.
    ///
    /// # Parameters
    ///
    /// - `char`: The character to render.
    /// - `position`: The top-left pixel position.
    /// - `color`: The color of the character.
    ///
    /// # Behavior
    ///
    /// If the character does not exist in the font set,
    /// nothing is drawn.
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

    /// Clears the entire screen to a single color.
    ///
    /// This method iterates over every pixel in the framebuffer
    /// and sets it to the provided color.
    ///
    /// # Parameters
    ///
    /// - `color`: The fill color.
    pub fn clear_screen(&mut self, color: Color) {
        let format = self.info.pixel_format;
        for pixel in self.get_pixels_mut() {
            Self::draw_pixel(pixel, format, color);
        }
    }

    /// Sets a single pixel at the given position.
    ///
    /// # Parameters
    ///
    /// - `point`: The pixel coordinates.
    /// - `color`: The pixel color.
    ///
    /// # Behavior
    ///
    /// If the coordinates are outside the framebuffer bounds,
    /// the function returns without modifying memory.
    pub fn set_pixel(&mut self, point: Point, color: Color) {
        let is_off_screen = point.x >= self.info.width || point.y >= self.info.height;
        if is_off_screen {
            return;
        }

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let stride = self.info.stride;

        let pixel_index = point.y * stride + point.x;
        let byte_offset = pixel_index * bytes_per_pixel;

        let buffer = self.framebuffer.buffer_mut();
        let pixel = &mut buffer[byte_offset..byte_offset + bytes_per_pixel];

        Self::draw_pixel(pixel, self.info.pixel_format, color);
    }

    /// Returns a mutable iterator over all pixels in the framebuffer.
    ///
    /// Each item in the iterator is a mutable slice representing
    /// one pixel (`bytes_per_pixel` bytes).
    fn get_pixels_mut(&mut self) -> alloc::slice::ChunksExactMut<'_, u8> {
        self.framebuffer
            .buffer_mut()
            .chunks_exact_mut(self.info.bytes_per_pixel)
    }

    /// Writes a color into a raw pixel slice according to pixel format.
    ///
    /// # Parameters
    ///
    /// - `pixel`: Mutable slice representing a single pixel.
    /// - `format`: Framebuffer pixel format.
    /// - `color`: Color to write.
    ///
    /// # Supported Formats
    ///
    /// - `Rgb`: Red, Green, Blue byte order.
    /// - `Bgr`: Blue, Green, Red byte order.
    /// - `U8`: Single-byte grayscale (uses red component).
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

/// A 2D coordinate in pixel space.
///
/// Represents a position in the framebuffer.
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// Horizontal position (0 is left).
    pub x: usize,

    /// Vertical position (0 is top).
    pub y: usize,
}

/// A simple RGB color representation.
///
/// Each component ranges from `0` to `255`.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    /// Red component.
    pub red: u8,

    /// Green component.
    pub green: u8,

    /// Blue component.
    pub blue: u8,
}

/// Returns `true` if the `n`th bit of `num` is set.
///
/// # Parameters
///
/// - `num`: The byte to inspect.
/// - `n`: Bit index (0 = least significant bit).
///
/// # Returns
///
/// `true` if the bit is 1, otherwise `false`.
#[inline]
fn n_th_bit_is_set(num: u8, n: u8) -> bool {
    (num >> n) & 1 == 1
}
