use image::{Rgb, RgbImage};

use crate::{PsfFont, VgaFont};

const CHARS_PER_ROW: u32 = 32;
const PADDING: u32 = 2;
const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const WHITE: Rgb<u8> = Rgb([255, 255, 255]);

impl PsfFont {
    pub fn preview(&self) -> RgbImage {
        render_preview(&self.glyphs, self.glyph_size)
    }
}

impl VgaFont {
    pub fn preview(self) -> RgbImage {
        render_preview(&self.glyphs, (8, self.height))
    }
}

fn render_preview(glyphs: &[impl AsRef<[u8]>], glyph_size: (u8, u8)) -> RgbImage {
    let cell_width = u32::from(glyph_size.0) + 2 * PADDING;
    let cell_height = u32::from(glyph_size.1) + 2 * PADDING;

    #[expect(clippy::cast_possible_truncation)]
    let rows = glyphs.len().div_ceil(CHARS_PER_ROW as usize) as u32;

    let img_width = CHARS_PER_ROW * cell_width + 2 * PADDING;
    let img_height = rows * cell_height + 2 * PADDING;
    let mut img = RgbImage::from_pixel(img_width, img_height, BLACK);

    for (index, glyph) in glyphs.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation)]
        let (row, col) = (index as u32 / CHARS_PER_ROW, index as u32 % CHARS_PER_ROW);
        let pos = (
            PADDING + col * cell_width + PADDING,
            PADDING + row * cell_height + PADDING,
        );
        draw_glyph(&mut img, glyph.as_ref(), glyph_size, pos);
    }

    img
}

fn draw_glyph(img: &mut RgbImage, glyph: &[u8], glyph_size: (u8, u8), pos: (u32, u32)) {
    let (glyph_width, glyph_height) = (u32::from(glyph_size.0), u32::from(glyph_size.1));
    let bytes_per_row = glyph_width.div_ceil(8);

    for row in 0..glyph_height {
        for col in 0..glyph_width {
            let byte_index = (row * bytes_per_row + col / 8) as usize;
            let bit_index = 7 - (col % 8);

            if let Some(&byte) = glyph.get(byte_index) {
                let color = if (byte >> bit_index) & 1 == 1 {
                    WHITE
                } else {
                    BLACK
                };
                img.put_pixel(pos.0 + col, pos.1 + row, color);
            }
        }
    }
}
