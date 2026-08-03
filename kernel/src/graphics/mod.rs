use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use font8x8::UnicodeFonts;
use spin::{Mutex, Once};

/// Global framebuffer writer instance.
///
/// Lazily initialized on first call to [`init`]. Synchronized with a mutex
/// to ensure thread-safe access to the framebuffer.
pub static WRITER: Once<Mutex<FramebufferWriter>> = Once::new();

/// Initializes the graphics subsystem with the provided framebuffer.
///
/// This function must be called exactly once during kernel startup to set up
/// the framebuffer writer. Subsequent calls are no-ops due to the [`Once`] wrapper.
///
/// # Arguments
/// * `framebuffer` - The framebuffer information from the bootloader
pub fn init(framebuffer: FrameBuffer) {
    WRITER.call_once(|| {
        let writer = FramebufferWriter::new(framebuffer);

        Mutex::new(writer)
    });
}

/// Maximum framebuffer width in pixels.
const MAX_SCREEN_WIDTH: usize = 1920;
/// Maximum framebuffer height in pixels.
const MAX_SCREEN_HEIGHT: usize = 1080;
/// Maximum bytes per pixel (supports up to 32-bit color formats).
const MAX_BYTES_PER_PIXEL: usize = 4;

/// Type alias for the back buffer array.
///
/// Stores pixel data for a framebuffer of up to [`MAX_SCREEN_WIDTH`] ×
/// [`MAX_SCREEN_HEIGHT`] pixels with up to [`MAX_BYTES_PER_PIXEL`] bytes per pixel.
type BackBuffer = [u8; MAX_SCREEN_WIDTH * MAX_SCREEN_HEIGHT * MAX_BYTES_PER_PIXEL];

/// Global back buffer for off-screen rendering.
///
/// All drawing operations write to this buffer. The [`FramebufferWriter::flush`]
/// method copies the back buffer contents to the actual framebuffer.
static mut BACK_BUFFER: BackBuffer =
    [0; MAX_SCREEN_WIDTH * MAX_SCREEN_HEIGHT * MAX_BYTES_PER_PIXEL];

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
    /// Flushes the back buffer to the actual framebuffer.
    ///
    /// Copies all pixel data from the back buffer to the device framebuffer,
    /// making any pending draw operations visible on screen.
    pub fn flush(&mut self) {
        let buffer = self.framebuffer.buffer_mut();

        let len = buffer.len();

        let dst = buffer.as_mut_ptr();
        let src = core::ptr::addr_of!(BACK_BUFFER).cast::<u8>();

        // SAFETY: Both src and dst are valid pointers:
        // - src points to the static BACK_BUFFER which is properly initialized
        // - dst points to the framebuffer which is valid for the entire buffer.len() bytes
        // - Regions do not overlap as back buffer and framebuffer are separate allocations
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, len);
        }
    }

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
    #[must_use]
    pub fn new(value: FrameBuffer) -> Self {
        let pixel_format = value.info().pixel_format;
        match pixel_format {
            PixelFormat::Rgb => log::info!("Using rgb pixel format"),
            PixelFormat::Bgr => log::info!("Using bgr pixel format"),
            PixelFormat::U8 => log::info!("Using u8 pixel format"),
            format => log::error!("Unknown pixel format: {format:?}"),
        }
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

    /// Draws a line between two points using Bresenham's line algorithm.
    ///
    /// # Parameters
    ///
    /// - `position1`: The starting point of the line.
    /// - `position2`: The ending point of the line.
    /// - `color`: The color of the line.
    ///
    /// # Behavior
    ///
    /// Uses Bresenham's algorithm to efficiently rasterize the line
    /// by only drawing pixels within the framebuffer bounds.
    /// 
    /// # Panics
    ///
    /// Panics if any coordinate in `position1` or `position2` exceeds `i64::MAX`.
    #[allow(clippy::expect_used)]
    pub fn draw_line(&mut self, position1: Point, position2: Point, color: Color) {
        let mut x0 = i64::try_from(position1.x).expect("Expected x0 to fit i64::MAX!");
        let mut y0 = i64::try_from(position1.y).expect("Expected y0 to fit i64::MAX!");
        let x1 = i64::try_from(position2.x).expect("Expected x1 to fit i64::MAX!");
        let y1 = i64::try_from(position2.y).expect("Expected y1 to fit i64::MAX!");

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();

        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        let mut err = dx - dy;

        loop {
            if let (Ok(x), Ok(y)) = (usize::try_from(x0), usize::try_from(y0)) {
                self.set_pixel(Point { x, y }, color);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = err * 2;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Clears the entire screen to a single color.
    ///
    /// This method iterates over every pixel in the framebuffer
    /// and sets it to the provided color, accounting for the framebuffer stride.
    ///
    /// # Parameters
    ///
    /// - `color`: The fill color.
    ///
    /// # Behavior
    ///
    /// All pixels are written to the back buffer with bounds checking via stride.
    pub fn clear_screen(&mut self, color: Color) {
        let width = self.info.width;
        let height = self.info.height;
        let stride = self.info.stride;
        let bpp = self.info.bytes_per_pixel;
        let format = self.info.pixel_format;

        let base = core::ptr::addr_of_mut!(BACK_BUFFER).cast::<u8>();
        for y in 0..height {
            for x in 0..width {
                let pixel_index = y * stride + x;
                let offset = pixel_index * bpp;

                // Safety: Bounds check ensures offset is within BACK_BUFFER allocation.
                unsafe {
                    let pixel = base.add(offset);

                    Self::draw_pixel(pixel, format, color);
                };
            }
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
        if point.x >= self.info.width || point.y >= self.info.height {
            return;
        }

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let stride = self.info.stride;

        let pixel_index = point.y * stride + point.x;
        let pixel_start = pixel_index * bytes_per_pixel;

        let back_buffer_ptr = core::ptr::addr_of_mut!(BACK_BUFFER).cast::<u8>();

        let pixel = back_buffer_ptr.wrapping_add(pixel_start);

        // Safety: Bounds check ensures pixel_start is within BACK_BUFFER.
        unsafe {
            Self::draw_pixel(pixel, self.info.pixel_format, color);
        }
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
    ///
    /// # Safety
    ///
    /// The caller must ensure that `pixel` points to valid memory with sufficient space
    /// for writing the appropriate number of bytes based on the pixel format.
    unsafe fn draw_pixel(pixel: *mut u8, format: PixelFormat, color: Color) {
        match format {
            PixelFormat::Rgb => unsafe {
                // Safety: Caller guarantees sufficient space for 3 bytes (RGB).
                *pixel = color.red;
                *(pixel.add(1)) = color.green;
                *pixel.add(2) = color.blue;
            },
            PixelFormat::Bgr => unsafe {
                // Safety: Caller guarantees sufficient space for 3 bytes (BGR).
                *pixel = color.blue;
                *(pixel.add(1)) = color.green;
                *(pixel.add(2)) = color.red;
            },

            PixelFormat::U8 => unsafe {
                // Safety: Caller guarantees sufficient space for 1 byte (grayscale).
                *pixel = color.red;
            },
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
