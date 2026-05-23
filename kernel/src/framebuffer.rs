//! Minimal framebuffer text surface for M5.
//!
//! The module is boot-passive: callers provide the pixel storage, dimensions,
//! stride, and colors explicitly. No global framebuffer pointer is stored here.

pub const CELL_WIDTH: usize = 8;
pub const CELL_HEIGHT: usize = 8;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FramebufferError {
    ZeroDimensions,
    StrideTooSmall,
    BufferTooSmall,
}

pub struct TextSurface<'a> {
    pixels: &'a mut [u32],
    width: usize,
    height: usize,
    stride_pixels: usize,
    cursor_col: usize,
    cursor_row: usize,
    fg: u32,
    bg: u32,
}

impl<'a> TextSurface<'a> {
    pub fn new(
        pixels: &'a mut [u32],
        width: usize,
        height: usize,
        stride_pixels: usize,
        fg: u32,
        bg: u32,
    ) -> Result<Self, FramebufferError> {
        if width == 0 || height == 0 {
            return Err(FramebufferError::ZeroDimensions);
        }
        if stride_pixels < width {
            return Err(FramebufferError::StrideTooSmall);
        }
        let required = stride_pixels
            .checked_mul(height)
            .ok_or(FramebufferError::BufferTooSmall)?;
        if pixels.len() < required {
            return Err(FramebufferError::BufferTooSmall);
        }

        Ok(Self {
            pixels,
            width,
            height,
            stride_pixels,
            cursor_col: 0,
            cursor_row: 0,
            fg,
            bg,
        })
    }

    pub fn clear(&mut self) {
        for pixel in self.pixels.iter_mut() {
            *pixel = self.bg;
        }
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    pub fn write_str(&mut self, text: &str) {
        for byte in text.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn write_bytes_ascii(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_byte(*byte);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.newline();
            return;
        }

        let cell_cols = self.cell_cols();
        let cell_rows = self.cell_rows();
        if cell_cols == 0 || cell_rows == 0 {
            return;
        }

        if self.cursor_col >= cell_cols {
            self.newline();
        }
        if self.cursor_row >= cell_rows {
            return;
        }

        self.draw_cell(self.cursor_col, self.cursor_row, byte);
        self.cursor_col += 1;
    }

    pub const fn cursor(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_row)
    }

    pub const fn cell_cols(&self) -> usize {
        self.width / CELL_WIDTH
    }

    pub const fn cell_rows(&self) -> usize {
        self.height / CELL_HEIGHT
    }

    fn newline(&mut self) {
        let next =
            Self::cursor_after_byte(self.cursor_col, self.cursor_row, self.cell_cols(), b'\n');
        self.cursor_col = next.0;
        self.cursor_row = next.1;
    }

    fn draw_cell(&mut self, col: usize, row: usize, byte: u8) {
        let glyph = glyph_rows(byte);
        let base_x = col * CELL_WIDTH;
        let base_y = row * CELL_HEIGHT;
        let mut y = 0;
        while y < CELL_HEIGHT {
            let bits = glyph[y];
            let mut x = 0;
            while x < CELL_WIDTH {
                let mask = 1 << (7 - x);
                let color = if bits & mask != 0 { self.fg } else { self.bg };
                if let Some(index) = Self::pixel_index(
                    self.width,
                    self.height,
                    self.stride_pixels,
                    base_x + x,
                    base_y + y,
                ) {
                    self.pixels[index] = color;
                }
                x += 1;
            }
            y += 1;
        }
    }

    const fn pixel_index(
        width: usize,
        height: usize,
        stride_pixels: usize,
        x: usize,
        y: usize,
    ) -> Option<usize> {
        if x >= width || y >= height {
            return None;
        }
        match y.checked_mul(stride_pixels) {
            Some(row) => row.checked_add(x),
            None => None,
        }
    }

    const fn cell_offset_for(stride_pixels: usize, col: usize, row: usize) -> usize {
        row * CELL_HEIGHT * stride_pixels + col * CELL_WIDTH
    }

    const fn cursor_after_byte(
        col: usize,
        row: usize,
        cell_cols: usize,
        byte: u8,
    ) -> (usize, usize) {
        if byte == b'\n' || col + 1 >= cell_cols {
            (0, row + 1)
        } else {
            (col + 1, row)
        }
    }
}

#[allow(clippy::too_many_lines)]
fn glyph_rows(byte: u8) -> [u8; CELL_HEIGHT] {
    let c = if byte.is_ascii_lowercase() {
        byte.to_ascii_uppercase()
    } else {
        byte
    };
    match c {
        b'0' => pack([
            " ### ", "#   #", "#  ##", "# # #", "##  #", "#   #", " ### ",
        ]),
        b'1' => pack([
            "  #  ", " ##  ", "# #  ", "  #  ", "  #  ", "  #  ", "#####",
        ]),
        b'2' => pack([
            " ### ", "#   #", "    #", "   # ", "  #  ", " #   ", "#####",
        ]),
        b'3' => pack([
            "#### ", "    #", "    #", " ### ", "    #", "    #", "#### ",
        ]),
        b'4' => pack([
            "#   #", "#   #", "#   #", "#####", "    #", "    #", "    #",
        ]),
        b'5' => pack([
            "#####", "#    ", "#    ", "#### ", "    #", "    #", "#### ",
        ]),
        b'6' => pack([
            " ### ", "#    ", "#    ", "#### ", "#   #", "#   #", " ### ",
        ]),
        b'7' => pack([
            "#####", "    #", "   # ", "  #  ", " #   ", " #   ", " #   ",
        ]),
        b'8' => pack([
            " ### ", "#   #", "#   #", " ### ", "#   #", "#   #", " ### ",
        ]),
        b'9' => pack([
            " ### ", "#   #", "#   #", " ####", "    #", "    #", " ### ",
        ]),
        b'A' => pack([
            " ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ]),
        b'B' => pack([
            "#### ", "#   #", "#   #", "#### ", "#   #", "#   #", "#### ",
        ]),
        b'C' => pack([
            " ### ", "#   #", "#    ", "#    ", "#    ", "#   #", " ### ",
        ]),
        b'D' => pack([
            "#### ", "#   #", "#   #", "#   #", "#   #", "#   #", "#### ",
        ]),
        b'E' => pack([
            "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#####",
        ]),
        b'F' => pack([
            "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#    ",
        ]),
        b'G' => pack([
            " ### ", "#   #", "#    ", "# ###", "#   #", "#   #", " ### ",
        ]),
        b'H' => pack([
            "#   #", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ]),
        b'I' => pack([
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "#####",
        ]),
        b'J' => pack([
            "#####", "   # ", "   # ", "   # ", "   # ", "#  # ", " ##  ",
        ]),
        b'K' => pack([
            "#   #", "#  # ", "# #  ", "##   ", "# #  ", "#  # ", "#   #",
        ]),
        b'L' => pack([
            "#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####",
        ]),
        b'M' => pack([
            "#   #", "## ##", "# # #", "#   #", "#   #", "#   #", "#   #",
        ]),
        b'N' => pack([
            "#   #", "##  #", "# # #", "#  ##", "#   #", "#   #", "#   #",
        ]),
        b'O' => pack([
            " ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ]),
        b'P' => pack([
            "#### ", "#   #", "#   #", "#### ", "#    ", "#    ", "#    ",
        ]),
        b'Q' => pack([
            " ### ", "#   #", "#   #", "#   #", "# # #", "#  # ", " ## #",
        ]),
        b'R' => pack([
            "#### ", "#   #", "#   #", "#### ", "# #  ", "#  # ", "#   #",
        ]),
        b'S' => pack([
            " ####", "#    ", "#    ", " ### ", "    #", "    #", "#### ",
        ]),
        b'T' => pack([
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ",
        ]),
        b'U' => pack([
            "#   #", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ]),
        b'V' => pack([
            "#   #", "#   #", "#   #", "#   #", "#   #", " # # ", "  #  ",
        ]),
        b'W' => pack([
            "#   #", "#   #", "#   #", "# # #", "# # #", "## ##", "#   #",
        ]),
        b'X' => pack([
            "#   #", "#   #", " # # ", "  #  ", " # # ", "#   #", "#   #",
        ]),
        b'Y' => pack([
            "#   #", "#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  ",
        ]),
        b'Z' => pack([
            "#####", "    #", "   # ", "  #  ", " #   ", "#    ", "#####",
        ]),
        b'_' => pack([
            "     ", "     ", "     ", "     ", "     ", "     ", "#####",
        ]),
        b'-' => pack([
            "     ", "     ", "     ", " ### ", "     ", "     ", "     ",
        ]),
        b'=' => pack([
            "     ", "#####", "     ", "#####", "     ", "     ", "     ",
        ]),
        b':' => pack([
            "     ", "  #  ", "     ", "     ", "  #  ", "     ", "     ",
        ]),
        b'>' => pack([
            "#    ", " #   ", "  #  ", "   # ", "  #  ", " #   ", "#    ",
        ]),
        b' ' => [0; CELL_HEIGHT],
        _ => pack([
            "#####", "#   #", "   # ", "  #  ", "     ", "  #  ", "     ",
        ]),
    }
}

fn pack(rows: [&str; 7]) -> [u8; CELL_HEIGHT] {
    let mut out = [0u8; CELL_HEIGHT];
    let mut y = 0;
    while y < 7 {
        let bytes = rows[y].as_bytes();
        let mut x = 0;
        while x < 5 {
            if bytes[x] != b' ' {
                out[y] |= 1 << (6 - x);
            }
            x += 1;
        }
        y += 1;
    }
    out
}

const _: () = assert!(TextSurface::cell_offset_for(10, 2, 1) == 96);
const _: () = assert!(cursor_is(
    TextSurface::cursor_after_byte(3, 0, 4, b'X'),
    0,
    1
));
const _: () = assert!(cursor_is(
    TextSurface::cursor_after_byte(1, 2, 4, b'\n'),
    0,
    3
));
const _: () = assert!(pixel_is(TextSurface::pixel_index(16, 8, 16, 15, 7), 127));
const _: () = assert!(pixel_is_none(TextSurface::pixel_index(16, 8, 16, 16, 7)));

const fn cursor_is(cursor: (usize, usize), col: usize, row: usize) -> bool {
    cursor.0 == col && cursor.1 == row
}

const fn pixel_is(pixel: Option<usize>, expected: usize) -> bool {
    match pixel {
        Some(index) => index == expected,
        None => false,
    }
}

#[allow(clippy::match_like_matches_macro, clippy::redundant_pattern_matching)]
const fn pixel_is_none(pixel: Option<usize>) -> bool {
    if let Some(_) = pixel {
        false
    } else {
        true
    }
}
