use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek as _, SeekFrom},
    str,
};

use color_eyre::{
    Result,
    eyre::{bail, eyre},
};
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, Immutable, KnownLayout)]
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

    const fn glyph_count(self) -> u16 {
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
    const ENTRY_END: u16 = 0xffff;
    const SEQUENCE_START: u16 = 0xfffe;

    pub fn from_file(mut file: File) -> Result<Self> {
        let header = Psf1Header::from_reader(&mut file)?;

        if header.magic != Self::MAGIC {
            bail!("Invalid PSF1 magic number");
        }

        let glyph_count: usize = if header.mode & 0x01 != 0 { 512 } else { 256 };
        let glyph_size = header.glyph_size as usize;

        let mut glyph_data = vec![0u8; header.glyph_count() as usize * glyph_size];
        file.read_exact(&mut glyph_data)?;

        let glyphs = glyph_data
            .chunks_exact(glyph_size)
            .map(<[u8]>::to_vec)
            .collect();

        let unicode_table = header
            .has_unicode_table()
            .then(|| Self::read_unicode_table(&mut file, glyph_count))
            .transpose()?;

        Ok(Self {
            header,
            glyphs,
            unicode_table,
        })
    }

    fn read_unicode_table(file: &mut File, glyph_count: usize) -> Result<HashMap<char, u16>> {
        let mut ut_data = Vec::new();
        file.read_to_end(&mut ut_data)?;

        let mut mappings = HashMap::new();
        let mut offset = 0;

        for glyph_index in 0..glyph_count {
            while offset + 1 < ut_data.len() {
                let code = u16::from_le_bytes([ut_data[offset], ut_data[offset + 1]]);
                offset += 2;

                match code {
                    Self::ENTRY_END => break,
                    Self::SEQUENCE_START => {
                        while offset + 1 < ut_data.len() {
                            let seq_code =
                                u16::from_le_bytes([ut_data[offset], ut_data[offset + 1]]);
                            offset += 2;
                            if seq_code == Self::ENTRY_END {
                                break;
                            }
                        }
                        break;
                    },
                    code =>
                        if let Some(char) = char::from_u32(u32::from(code)) {
                            #[expect(clippy::cast_possible_truncation)]
                            mappings.insert(char, glyph_index as u16);
                        },
                }
            }
        }

        Ok(mappings)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes, Immutable, KnownLayout)]
pub struct Psf2Header {
    pub magic: u32,
    pub version: u32,
    pub header_size: u32,
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
    const MAGIC: u32 = 0x864a_b572;
    const ENTRY_END: u8 = 0xff;
    const SEQUENCE_START: u8 = 0xfe;

    pub fn from_file(mut file: File) -> Result<Self> {
        let header = Psf2Header::from_reader(&mut file)?;

        if header.magic != Self::MAGIC {
            bail!("Invalid PSF2 magic number");
        }
        if header.version != 0 {
            let version = header.version;
            bail!("Unsupported PSF2 version: `{}`", version);
        }

        let mut glyph_data = vec![0u8; (header.length * header.glyph_size) as usize];
        file.read_exact(&mut glyph_data)?;

        let glyphs = glyph_data
            .chunks_exact(header.glyph_size as usize)
            .map(<[u8]>::to_vec)
            .collect();

        let unicode_table = header
            .has_unicode_table()
            .then(|| Self::read_unicode_table(&mut file, header.length as usize))
            .transpose()?;

        Ok(Self {
            header,
            glyphs,
            unicode_table,
        })
    }

    fn read_unicode_table(file: &mut File, glyph_count: usize) -> Result<HashMap<char, u16>> {
        let mut ut_data = Vec::new();
        file.read_to_end(&mut ut_data)?;

        let mut mappings = HashMap::new();
        let mut offset = 0;

        for glyph_index in 0..glyph_count {
            while offset < ut_data.len() {
                match ut_data[offset] {
                    Self::ENTRY_END => {
                        offset += 1;
                        break;
                    },
                    Self::SEQUENCE_START => {
                        offset += 1;
                        while offset < ut_data.len() {
                            if ut_data[offset] == Self::ENTRY_END {
                                offset += 1;
                                break;
                            }
                            offset += 1;
                        }
                        break;
                    },
                    _ => {
                        let remaining = &ut_data[offset..];
                        let s = str::from_utf8(remaining)
                            .map_err(|_| eyre!("Invalid UTF-8 in unicode table"))?;

                        if let Some(ch) = s.chars().next() {
                            #[expect(clippy::cast_possible_truncation)]
                            mappings.insert(ch, glyph_index as u16);
                            offset += ch.len_utf8();
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
    pub fn from_file(mut file: File) -> Result<Self> {
        let mut peek_bytes = [0u8; 4];
        file.read_exact(&mut peek_bytes)?;
        file.seek(SeekFrom::Start(0))?;

        if u32::from_le_bytes(peek_bytes) == Psf2Font::MAGIC {
            return Ok(Psf2Font::from_file(file)?.into());
        }

        #[expect(clippy::unwrap_used)]
        if u16::from_le_bytes(peek_bytes[..2].try_into().unwrap()) == Psf1Font::MAGIC {
            return Ok(Psf1Font::from_file(file)?.into());
        }

        bail!("File is neither PSF1 nor PSF2 format")
    }
}
