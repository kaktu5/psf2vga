use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead as _, BufReader, Cursor, Read, Seek},
    str,
};

use color_eyre::{
    Result,
    eyre::{bail, eyre},
};
use flate2::read::GzDecoder;
use image::{Rgb, RgbImage};
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(Clone, Copy, FromBytes, Immutable, KnownLayout)]
pub struct Psf1Header {
    pub magic: u16,
    pub mode: u8,
    pub glyph_size: u8,
}

impl Psf1Header {
    const HAS_512_GLYPHS: u8 = 0x01;
    const HAS_UNICODE_TABLE: u8 = 0x02 | 0x04;

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Self::read_from_bytes(&buf[..]).map_err(|_| eyre!("Invalid PSF1 header format"))
    }

    const fn glyph_count(self) -> usize {
        if self.mode & Self::HAS_512_GLYPHS != 0 {
            512
        } else {
            256
        }
    }

    const fn has_unicode_table(self) -> bool {
        self.mode & Self::HAS_UNICODE_TABLE != 0
    }
}

pub struct Psf1Font {
    pub header: Psf1Header,
    pub glyphs: Vec<Vec<u8>>,
    pub unicode_table: Option<HashMap<char, u16>>,
}

impl Psf1Font {
    const MAGIC: u16 = 0x0436;
    const ENTRY_END: u16 = 0xFFFF;
    const SEQUENCE_START: u16 = 0xFFFE;

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let header = Psf1Header::from_reader(reader)?;

        if header.magic != Self::MAGIC {
            bail!("Invalid PSF1 magic number");
        }

        let glyph_size = header.glyph_size as usize;

        let mut glyph_data = vec![0u8; header.glyph_count() * glyph_size];
        reader.read_exact(&mut glyph_data)?;

        let glyphs = glyph_data
            .chunks_exact(glyph_size)
            .map(<[u8]>::to_vec)
            .collect();

        let unicode_table = header
            .has_unicode_table()
            .then(|| Self::read_unicode_table(reader, header.glyph_count()))
            .transpose()?;

        Ok(Self {
            header,
            glyphs,
            unicode_table,
        })
    }

    fn read_unicode_table<R: Read>(
        reader: &mut R,
        glyph_count: usize,
    ) -> Result<HashMap<char, u16>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let mut mappings = HashMap::new();
        let mut codes = data
            .chunks(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));

        for glyph_index in 0..glyph_count {
            for code in codes.by_ref() {
                match code {
                    Self::ENTRY_END => break,
                    Self::SEQUENCE_START => {
                        codes
                            .by_ref()
                            .take_while(|&c| c != Self::ENTRY_END)
                            .for_each(drop);
                        break;
                    },
                    code =>
                        if let Some(ch) = char::from_u32(u32::from(code)) {
                            #[expect(clippy::cast_possible_truncation)]
                            mappings.insert(ch, glyph_index as u16);
                        },
                }
            }
        }

        Ok(mappings)
    }
}

#[derive(Clone, Copy, FromBytes, Immutable, KnownLayout)]
pub struct Psf2Header {
    pub magic: u32,
    pub version: u32,
    pub _header_size: u32,
    pub flags: u32,
    pub length: u32,
    pub glyph_size: u32,
    pub height: u32,
    pub width: u32,
}

impl Psf2Header {
    const HAS_UNICODE_TABLE: u32 = 0x0000_0001;

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Self::read_from_bytes(&buf[..]).map_err(|_| eyre!("Invalid PSF2 header format"))
    }

    const fn glyph_count(self) -> usize {
        self.length as usize
    }

    const fn has_unicode_table(self) -> bool {
        self.flags & Self::HAS_UNICODE_TABLE != 0
    }
}

pub struct Psf2Font {
    pub header: Psf2Header,
    pub glyphs: Vec<Vec<u8>>,
    pub unicode_table: Option<HashMap<char, u16>>,
}

impl Psf2Font {
    const MAGIC: u32 = 0x864A_B572;
    const ENTRY_END: u8 = 0xFF;
    const SEQUENCE_START: u8 = 0xFE;

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let header = Psf2Header::from_reader(reader)?;

        if header.magic != Self::MAGIC {
            bail!("Invalid PSF2 magic number");
        }
        if header.version != 0 {
            let version = header.version;
            bail!("Unsupported PSF2 version: {}", version);
        }

        let mut glyph_data = vec![0u8; (header.length * header.glyph_size) as usize];
        reader.read_exact(&mut glyph_data)?;

        let glyphs = glyph_data
            .chunks_exact(header.glyph_size as usize)
            .map(<[u8]>::to_vec)
            .collect();

        let unicode_table = header
            .has_unicode_table()
            .then(|| Self::read_unicode_table(reader, header.glyph_count()))
            .transpose()?;

        Ok(Self {
            header,
            glyphs,
            unicode_table,
        })
    }

    fn read_unicode_table<R: Read>(
        reader: &mut R,
        glyph_count: usize,
    ) -> Result<HashMap<char, u16>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let mut mappings = HashMap::new();
        let mut bytes = data.iter().copied().peekable();

        for glyph_index in 0..glyph_count {
            while let Some(&byte) = bytes.peek() {
                match byte {
                    Self::ENTRY_END => {
                        bytes.next();
                        break;
                    },
                    Self::SEQUENCE_START => {
                        bytes.next();
                        bytes
                            .by_ref()
                            .take_while(|&b| b != Self::ENTRY_END)
                            .for_each(drop);
                        bytes.next();
                        break;
                    },
                    _ => {
                        let remaining: Vec<u8> = bytes.clone().collect();
                        let s = str::from_utf8(&remaining)
                            .map_err(|_| eyre!("Invalid UTF-8 in unicode table"))?;

                        if let Some(ch) = s.chars().next() {
                            #[expect(clippy::cast_possible_truncation)]
                            mappings.insert(ch, glyph_index as u16);
                            for _ in 0..ch.len_utf8() {
                                bytes.next();
                            }
                        } else {
                            break;
                        }
                    },
                }
            }
        }

        Ok(mappings)
    }
}

pub struct PsfFont {
    pub glyph_size: (u8, u8),
    pub glyphs: Vec<Vec<u8>>,
    pub unicode_table: Option<HashMap<char, u16>>,
}

impl From<Psf1Font> for PsfFont {
    fn from(font: Psf1Font) -> Self {
        Self {
            glyph_size: (8, font.header.glyph_size),
            glyphs: font.glyphs,
            unicode_table: font.unicode_table,
        }
    }
}

impl From<Psf2Font> for PsfFont {
    fn from(font: Psf2Font) -> Self {
        Self {
            #[expect(clippy::cast_possible_truncation)]
            glyph_size: (font.header.width as u8, font.header.height as u8),
            glyphs: font.glyphs,
            unicode_table: font.unicode_table,
        }
    }
}

impl PsfFont {
    const GZIP_MAGIC: u16 = 0x8B1F;
    const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
    const WHITE: Rgb<u8> = Rgb([255, 255, 255]);

    pub fn from_file(file: File) -> Result<Self> {
        let mut reader = BufReader::new(file);

        let header = reader.fill_buf()?;
        if header.len() >= 2 && u16::from_le_bytes([header[0], header[1]]) == Self::GZIP_MAGIC {
            let mut decoder = GzDecoder::new(reader);
            let mut data = Vec::new();
            decoder.read_to_end(&mut data)?;

            return Self::from_reader(Cursor::new(data));
        }

        Self::from_reader(reader)
    }

    pub fn from_reader<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let mut header_bytes = [0u8; 4];
        reader.read_exact(&mut header_bytes)?;
        reader.rewind()?;

        if u32::from_le_bytes(header_bytes) == Psf2Font::MAGIC {
            return Ok(Psf2Font::from_reader(&mut reader)?.into());
        }

        if u16::from_le_bytes([header_bytes[0], header_bytes[1]]) == Psf1Font::MAGIC {
            return Ok(Psf1Font::from_reader(&mut reader)?.into());
        }

        bail!("File is neither PSF1 nor PSF2 format")
    }

    pub fn render_preview(&self) -> RgbImage {
        const CHARS_PER_ROW: u32 = 32;
        const PADDING: u32 = 2;

        let cell_width = u32::from(self.glyph_size.0) + 2 * PADDING;
        let cell_height = u32::from(self.glyph_size.1) + 2 * PADDING;

        #[expect(clippy::cast_possible_truncation)]
        let rows = self.glyphs.len().div_ceil(CHARS_PER_ROW as usize) as u32;

        let img_width = CHARS_PER_ROW * cell_width + 2 * PADDING;
        let img_height = rows * cell_height + 2 * PADDING;
        let mut img = RgbImage::from_pixel(img_width, img_height, Self::BLACK);

        for (index, glyph) in self.glyphs.iter().enumerate() {
            #[expect(clippy::cast_possible_truncation)]
            let (row, col) = (index as u32 / CHARS_PER_ROW, index as u32 % CHARS_PER_ROW);
            let pos = (
                PADDING + col * cell_width + PADDING,
                PADDING + row * cell_height + PADDING,
            );
            self.draw_glyph(&mut img, glyph, pos);
        }

        img
    }

    fn draw_glyph(&self, img: &mut RgbImage, glyph: &[u8], pos: (u32, u32)) {
        let (glyph_width, glyph_height) =
            (u32::from(self.glyph_size.0), u32::from(self.glyph_size.1));
        let bytes_per_row = glyph_width.div_ceil(8);

        for row in 0..glyph_height {
            for col in 0..glyph_width {
                let byte_index = (row * bytes_per_row + col / 8) as usize;
                let bit_index = 7 - (col % 8);

                if let Some(&byte) = glyph.get(byte_index) {
                    let color = if (byte >> bit_index) & 1 == 1 {
                        Self::WHITE
                    } else {
                        Self::BLACK
                    };
                    img.put_pixel(pos.0 + col, pos.1 + row, color);
                }
            }
        }
    }
}
