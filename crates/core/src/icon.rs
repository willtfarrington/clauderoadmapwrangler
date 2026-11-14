//! The app's own icon (D-21, D-29; EP-15, EP-28a): a blue tile with a roadmap's three white rows,
//! drawn by the tool itself at any size, so that no picture ships beside the code. The tray draws
//! it with its badges ([`draw`]): a red disc with the count of what waits for the owner, and an
//! amber dot while a badge event is new. The window takes the bare tile as its icon ([`plain`]),
//! and `crw install` writes it as a Windows `.ico` beside the exes for the shortcuts ([`ico`]),
//! one image a size, each drawn at its own size rather than scaled.

/// What the icon's pixels show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub waiting: usize,
    pub new: bool,
    pub size: u32,
}

const BLUE: [u8; 3] = [37, 99, 235];
const WHITE: [u8; 3] = [255, 255, 255];
const RED: [u8; 3] = [220, 38, 38];
const AMBER: [u8; 3] = [245, 158, 11];

/// The sides the shortcuts' `.ico` holds, from Windows' small icon to Explorer's largest.
pub const ICO_SIZES: [u32; 10] = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256];

/// The `.ico`'s name beside the installed exes.
pub const ICO_FILE: &str = "app.ico";

/// The count's digits and its plus, three pixels wide and five high: each row's three bits, the
/// leftmost the highest.
const GLYPHS: [(char, [u8; 5]); 11] = [
    ('0', [0b111, 0b101, 0b101, 0b101, 0b111]),
    ('1', [0b010, 0b110, 0b010, 0b010, 0b111]),
    ('2', [0b111, 0b001, 0b111, 0b100, 0b111]),
    ('3', [0b111, 0b001, 0b111, 0b001, 0b111]),
    ('4', [0b101, 0b101, 0b111, 0b001, 0b001]),
    ('5', [0b111, 0b100, 0b111, 0b001, 0b111]),
    ('6', [0b111, 0b100, 0b111, 0b101, 0b111]),
    ('7', [0b111, 0b001, 0b001, 0b001, 0b001]),
    ('8', [0b111, 0b101, 0b111, 0b101, 0b111]),
    ('9', [0b111, 0b101, 0b111, 0b001, 0b111]),
    ('+', [0b000, 0b010, 0b111, 0b010, 0b000]),
];

/// The icon's pixels for `look`, RGBA with straight alpha, `look.size` square.
pub fn draw(look: &Look) -> Vec<u8> {
    let s = look.size as f32;
    let mut canvas = Canvas::new(look.size);
    canvas.rounded([0.06 * s, 0.06 * s, 0.94 * s, 0.94 * s], 0.22 * s, BLUE);
    let half = 0.055 * s;
    for (y, length) in [(0.32, 0.50), (0.50, 0.38), (0.68, 0.26)] {
        let rect = [0.24 * s, y * s - half, (0.24 + length) * s, y * s + half];
        canvas.rounded(rect, half, WHITE);
    }
    let ring = (s / 24.0).max(1.0);
    if look.new {
        let (x, y, r) = (0.80 * s, 0.20 * s, 0.17 * s);
        canvas.disc(x, y, r + ring, WHITE);
        canvas.disc(x, y, r, AMBER);
    }
    if look.waiting > 0 {
        let r = 0.30 * s;
        let centre = s - r - 0.02 * s;
        canvas.disc(centre, centre, r + ring, WHITE);
        canvas.disc(centre, centre, r, RED);
        let count = if look.waiting > 9 {
            "9+".to_owned()
        } else {
            look.waiting.to_string()
        };
        canvas.text(&count, centre, centre, 2.0 * (r - ring), WHITE);
    }
    canvas.pixels
}

/// The bare tile, `size` square: the icon with nothing waiting and nothing new.
pub fn plain(size: u32) -> Vec<u8> {
    draw(&Look {
        waiting: 0,
        new: false,
        size,
    })
}

/// A Windows icon file holding one 32-bit image for each of `sizes` (1 to 256), each drawn by
/// [`plain`] at its own size: the header, an entry for each image, then each image as a bitmap
/// (its header, its rows bottom-up in blue, green, red and alpha, then its mask).
pub fn ico(sizes: &[u32]) -> Vec<u8> {
    let sizes: Vec<u32> = sizes.iter().map(|size| (*size).clamp(1, 256)).collect();
    let images: Vec<Vec<u8>> = sizes.iter().map(|size| bitmap(*size)).collect();
    let mut out = Vec::new();
    // The header: reserved, an icon (1), the count.
    for word in [0u16, 1, images.len() as u16] {
        out.extend_from_slice(&word.to_le_bytes());
    }
    let mut offset = 6 + 16 * images.len();
    for (size, image) in sizes.iter().zip(&images) {
        // A side of 256 does not fit the entry's byte, which says it as 0.
        let side = u8::try_from(*size).unwrap_or(0);
        // Width, height, no palette, reserved; one plane of 32 bits.
        out.extend_from_slice(&[side, side, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(image.len() as u32).to_le_bytes());
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += image.len();
    }
    for image in images {
        out.extend(image);
    }
    out
}

/// One image of an `.ico`: a `BITMAPINFOHEADER` whose height counts the image and its mask, the
/// tile's rows bottom-up as BGRA, then the AND mask, bottom-up too, a bit set (the highest bit
/// first) for each fully clear pixel, so that a reader that falls back on the mask leaves the
/// corners clear; each mask row padded to four bytes.
fn bitmap(size: u32) -> Vec<u8> {
    let pixels = plain(size);
    let side = size as usize;
    let image = side * side * 4;
    let mask = side.div_ceil(32) * 4 * side;
    let mut out = Vec::with_capacity(40 + image + mask);
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(size as i32).to_le_bytes());
    out.extend_from_slice(&(2 * size as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    // BI_RGB, then the image's bytes with its mask's.
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&((image + mask) as u32).to_le_bytes());
    // No resolution and no palette.
    out.extend_from_slice(&[0; 16]);
    for row in pixels.chunks_exact(side * 4).rev() {
        for [r, g, b, a] in row.as_chunks::<4>().0 {
            out.extend_from_slice(&[*b, *g, *r, *a]);
        }
    }
    let stride = side.div_ceil(32) * 4;
    for row in pixels.chunks_exact(side * 4).rev() {
        let mut bits = vec![0u8; stride];
        for (x, [_, _, _, a]) in row.as_chunks::<4>().0.iter().enumerate() {
            if *a == 0 {
                bits[x / 8] |= 0x80 >> (x % 8);
            }
        }
        out.extend(bits);
    }
    out
}

/// Straight-alpha RGBA pixels, drawn with each pixel covered by how much of it a shape holds.
struct Canvas {
    size: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    fn new(size: u32) -> Self {
        Self {
            size,
            pixels: vec![0; (size * size * 4) as usize],
        }
    }

    /// Lays `rgb` over the pixel at `x`, `y` by `cover`, from 0 to 1.
    fn blend(&mut self, x: u32, y: u32, rgb: [u8; 3], cover: f32) {
        let a = cover.clamp(0.0, 1.0);
        if a == 0.0 {
            return;
        }
        let at = ((y * self.size + x) * 4) as usize;
        let below = f32::from(self.pixels[at + 3]) / 255.0;
        let alpha = a + below * (1.0 - a);
        for (channel, value) in rgb.into_iter().enumerate() {
            let was = f32::from(self.pixels[at + channel]);
            let mixed = (f32::from(value) * a + was * below * (1.0 - a)) / alpha;
            self.pixels[at + channel] = mixed.round() as u8;
        }
        self.pixels[at + 3] = (alpha * 255.0).round() as u8;
    }

    /// Fills `rgb` by `cover`, which gives how much of the pixel centred at `x`, `y` lies inside.
    fn fill(&mut self, rgb: [u8; 3], cover: impl Fn(f32, f32) -> f32) {
        for y in 0..self.size {
            for x in 0..self.size {
                let inside = cover(x as f32 + 0.5, y as f32 + 0.5);
                self.blend(x, y, rgb, inside);
            }
        }
    }

    fn disc(&mut self, cx: f32, cy: f32, r: f32, rgb: [u8; 3]) {
        self.fill(rgb, |x, y| r - (x - cx).hypot(y - cy) + 0.5);
    }

    /// A rectangle `[left, top, right, bottom]` with its corners rounded by `radius`.
    fn rounded(&mut self, rect: [f32; 4], radius: f32, rgb: [u8; 3]) {
        let [left, top, right, bottom] = rect;
        let (cx, cy) = ((left + right) / 2.0, (top + bottom) / 2.0);
        let (hx, hy) = ((right - left) / 2.0 - radius, (bottom - top) / 2.0 - radius);
        self.fill(rgb, |x, y| {
            let (qx, qy) = ((x - cx).abs() - hx, (y - cy).abs() - hy);
            let outside = qx.max(0.0).hypot(qy.max(0.0));
            let inside = qx.max(qy).min(0.0);
            0.5 - (outside + inside - radius)
        });
    }

    /// Writes `text` in [`GLYPHS`] centred on `cx`, `cy`: as large as fits in `width`, by whole
    /// pixels, a twelfth of the icon at most.
    fn text(&mut self, text: &str, cx: f32, cy: f32, width: f32, rgb: [u8; 3]) {
        let glyphs: Vec<[u8; 5]> = text
            .chars()
            .filter_map(|c| GLYPHS.iter().find(|(glyph, _)| *glyph == c))
            .map(|(_, rows)| *rows)
            .collect();
        let count = glyphs.len() as u32;
        if count == 0 {
            return;
        }
        let span = |scale: u32| count * 4 * scale - scale;
        let mut scale = (self.size / 12).max(1);
        while scale > 1 && span(scale) as f32 > width {
            scale -= 1;
        }
        let left = (cx - span(scale) as f32 / 2.0).round() as i64;
        let top = (cy - (5 * scale) as f32 / 2.0).round() as i64;
        let size = i64::from(self.size);
        for (n, rows) in glyphs.iter().enumerate() {
            for (row, bits) in rows.iter().enumerate() {
                for column in 0..3 {
                    if bits & (0b100 >> column) == 0 {
                        continue;
                    }
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let x = left + i64::from((n as u32 * 4 + column) * scale + dx);
                            let y = top + i64::from(row as u32 * scale + dy);
                            if (0..size).contains(&x) && (0..size).contains(&y) {
                                self.blend(x as u32, y as u32, rgb, 1.0);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(pixels: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let at = ((y * size + x) * 4) as usize;
        [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
    }

    fn opaque(rgb: [u8; 3]) -> [u8; 4] {
        [rgb[0], rgb[1], rgb[2], 255]
    }

    #[test]
    fn the_icon_is_a_tile_with_a_count_and_a_dot_when_asked() {
        let plain = draw(&Look {
            waiting: 0,
            new: false,
            size: 24,
        });
        assert_eq!(plain.len(), 24 * 24 * 4);
        // The corner lies outside the tile's rounding; above the rows, the tile.
        assert_eq!(pixel(&plain, 24, 0, 0)[3], 0);
        assert_eq!(pixel(&plain, 24, 12, 3), opaque(BLUE));
        assert_eq!(pixel(&plain, 24, 8, 7), opaque(WHITE));
        let has =
            |pixels: &[u8], rgb: [u8; 3]| pixels.chunks(4).any(|px| px == opaque(rgb).as_slice());
        assert!(!has(&plain, RED) && !has(&plain, AMBER));

        let marked = draw(&Look {
            waiting: 3,
            new: true,
            size: 24,
        });
        // The disc's centre is the 3's middle bar; beside the digit, the disc.
        assert_eq!(pixel(&marked, 24, 16, 16), opaque(WHITE));
        assert_eq!(pixel(&marked, 24, 11, 16), opaque(RED));
        assert_eq!(pixel(&marked, 24, 19, 4), opaque(AMBER));
        // Many waiting: 9+, its two glyphs inside the disc.
        let many = draw(&Look {
            waiting: 14,
            new: false,
            size: 16,
        });
        assert!(has(&many, RED));
        assert_eq!(many.len(), 16 * 16 * 4);
    }

    #[test]
    fn the_plain_tile_is_blue_with_white_rows_and_clear_corners() {
        let tile = plain(32);
        assert_eq!(tile.len(), 32 * 32 * 4);
        assert_eq!(pixel(&tile, 32, 0, 0)[3], 0);
        assert_eq!(pixel(&tile, 32, 31, 31)[3], 0);
        // Above the rows, the tile; on the first row, white.
        assert_eq!(pixel(&tile, 32, 16, 4), opaque(BLUE));
        assert_eq!(pixel(&tile, 32, 12, 10), opaque(WHITE));
        assert_eq!(
            tile,
            draw(&Look {
                waiting: 0,
                new: false,
                size: 32,
            })
        );
    }

    /// The header, the entries with their lengths and offsets, and each bitmap: its header, the
    /// tile's rows bottom-up as BGRA, its mask marking the clear pixels.
    #[test]
    fn an_ico_holds_one_bitmap_a_size_each_drawn_at_its_own() {
        let bytes = ico(&[16, 256]);
        let u16_at = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        let u32_at = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        assert_eq!((u16_at(0), u16_at(2), u16_at(4)), (0, 1, 2));
        let length = |side: usize| 40 + side * side * 4 + side.div_ceil(32) * 4 * side;
        assert_eq!((length(16), length(256)), (1128, 270_376));
        let entry = |n: usize| 6 + 16 * n;
        assert_eq!(&bytes[entry(0)..entry(0) + 4], &[16, 16, 0, 0]);
        // A side of 256 is written 0.
        assert_eq!(&bytes[entry(1)..entry(1) + 4], &[0, 0, 0, 0]);
        for n in 0..2 {
            assert_eq!((u16_at(entry(n) + 4), u16_at(entry(n) + 6)), (1, 32));
        }
        assert_eq!((u32_at(entry(0) + 8), u32_at(entry(0) + 12)), (1128, 38));
        assert_eq!(
            (u32_at(entry(1) + 8), u32_at(entry(1) + 12)),
            (270_376, 38 + 1128)
        );
        assert_eq!(bytes.len(), 38 + 1128 + 270_376);
        for (n, side) in [(0, 16usize), (1, 256)] {
            let at = u32_at(entry(n) + 12) as usize;
            let (image, mask) = (side * side * 4, side.div_ceil(32) * 4 * side);
            assert_eq!(u32_at(at), 40);
            assert_eq!(u32_at(at + 4) as usize, side);
            assert_eq!(u32_at(at + 8) as usize, 2 * side);
            assert_eq!((u16_at(at + 12), u16_at(at + 14)), (1, 32));
            assert_eq!(u32_at(at + 16), 0);
            assert_eq!(u32_at(at + 20) as usize, image + mask);
            assert!(bytes[at + 24..at + 40].iter().all(|byte| *byte == 0));
            let drawn = plain(side as u32);
            let stored = &bytes[at + 40..at + 40 + image];
            for y in 0..side {
                for x in 0..side {
                    let (from, to) = ((y * side + x) * 4, ((side - 1 - y) * side + x) * 4);
                    assert_eq!(
                        stored[to..to + 4],
                        [
                            drawn[from + 2],
                            drawn[from + 1],
                            drawn[from],
                            drawn[from + 3]
                        ],
                        "{side}: {x}, {y}"
                    );
                }
            }
            // The mask, bottom-up as the image: a bit set (the highest first) for each clear
            // pixel, as the tile's corners are, and none for an opaque one.
            let mask_bytes = &bytes[at + 40 + image..at + 40 + image + mask];
            let stride = side.div_ceil(32) * 4;
            for y in 0..side {
                for x in 0..side {
                    let clear = drawn[(y * side + x) * 4 + 3] == 0;
                    let byte = mask_bytes[(side - 1 - y) * stride + x / 8];
                    let set = byte & (0x80 >> (x % 8)) != 0;
                    assert_eq!(set, clear, "{side}: {x}, {y}");
                }
            }
            assert!(mask_bytes[0] & 0x80 != 0, "{side}: a corner is clear");
        }
        // The shortcuts' file holds every size.
        let all = ico(&ICO_SIZES);
        assert_eq!(u16::from_le_bytes([all[4], all[5]]), 10);
    }
}
