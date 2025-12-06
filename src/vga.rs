use std::collections::HashMap;

use color_eyre::{Report, Result, eyre::eyre};

use crate::{cp437::DECODE_TABLE, psf::PsfFont};

#[derive(Debug, Clone)]
pub struct VgaFont {
    pub height: u8,
    pub glyphs: [[u8; 32]; 256],
}

impl VgaFont {
    pub const fn new(height: u8) -> Self {
        assert!(height <= 32, "VGA font height cannot exceed 32 pixels");
        Self {
            height,
            glyphs: [[0; 32]; 256],
        }
    }
}

impl TryFrom<PsfFont> for VgaFont {
    type Error = Report;

    fn try_from(psf: PsfFont) -> Result<Self> {
        const QUESTION_MARK_IDX: usize = 0x3F;

        if psf.glyph_size.0 != 8 {
            return Err(eyre!(
                "PSF font must be 8 pixels wide for VGA, got {}",
                psf.glyph_size.0
            ));
        }

        let height = psf.glyph_size.1 as usize;
        let mut vga = Self::new(psf.glyph_size.1);

        let fallback_glyph: Vec<u8> = psf.glyphs[QUESTION_MARK_IDX]
            .iter()
            .map(|&byte| !byte)
            .collect();

        let unicode_table = psf
            .unicode_table
            .as_ref()
            .ok_or_else(|| eyre!("PSF font has no Unicode table"))?;

        for (cp437_code, &unicode_char) in DECODE_TABLE.iter().enumerate() {
            let glyph = find_glyph(&psf, unicode_table, unicode_char, &fallback_glyph);
            let copy_len = height.min(glyph.len());
            vga.glyphs[cp437_code][..copy_len].copy_from_slice(&glyph[..copy_len]);
        }

        Ok(vga)
    }
}

fn find_glyph<'a>(
    psf: &'a PsfFont,
    unicode_table: &HashMap<char, u16>,
    ch: char,
    fallback: &'a [u8],
) -> &'a [u8] {
    if let Some(&index) = unicode_table.get(&ch)
        && let Some(glyph) = psf.glyphs.get(index as usize)
    {
        return glyph;
    }

    if ch.is_ascii_alphanumeric() {
        let alternate_case = match ch {
            ch if ch.is_ascii_lowercase() => ch.to_ascii_uppercase(),
            ch if ch.is_ascii_uppercase() => ch.to_ascii_lowercase(),
            ch => ch,
        };

        if let Some(&index) = unicode_table.get(&alternate_case)
            && let Some(glyph) = psf.glyphs.get(index as usize)
        {
            return glyph;
        }
    }

    fallback
}
