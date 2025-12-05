use std::{
    fs::File,
    io::{Read, Seek as _},
    ptr,
};

use color_eyre::{Result, eyre::bail};

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Psf1Header {
    pub magic: u16,
    pub mode: u8,
    pub glyph_size: u8,
}

impl Psf1Header {
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let mut header_bytes = [0u8; 4];
        reader.read_exact(&mut header_bytes)?;

        Ok(unsafe { ptr::read_unaligned(header_bytes.as_ptr().cast()) })
    }
}

pub struct Psf1Font {
    pub header: Psf1Header,
    pub glyphs: Vec<Vec<u8>>,
}

impl Psf1Font {
    const MAGICS: [u16; 2] = [0x0436, 0x0001];

    pub fn from_file(mut file: File) -> Result<Self> {
        let header = Psf1Header::from_reader(&mut file)?;

        // creating a misaligned reference is undefined behavior
        let magic = header.magic;
        if !Self::MAGICS.contains(&magic) {
            bail!("Invalid PSF1 magic number: `{:x}`", magic);
        }

        let glyph_count = if (header.mode & 0x01) != 0 { 512 } else { 256 };
        let mut glyphs = Vec::with_capacity(glyph_count);

        for _ in 0..glyph_count {
            let mut glyph = vec![0u8; header.glyph_size as usize];
            file.read_exact(&mut glyph)?;
            glyphs.push(glyph);
        }

        Ok(Self { header, glyphs })
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
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
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let mut header_bytes = [0u8; 32];
        reader.read_exact(&mut header_bytes)?;

        Ok(unsafe { ptr::read_unaligned(header_bytes.as_ptr().cast()) })
    }
}

pub struct Psf2Font {
    pub header: Psf2Header,
    pub glyphs: Vec<Vec<u8>>,
    pub unicode_table: Option<Vec<(u32, u32)>>,
}

impl Psf2Font {
    const MAGIC: u32 = 0x864a_b572;

    pub fn from_file(mut file: File) -> Result<Self> {
        let header = Psf2Header::from_reader(&mut file)?;

        if header.magic != Self::MAGIC {
            // creating a misaligned reference is undefined behavior
            let magic = header.magic;
            bail!("Invalid PSF2 magic number: `{:x}`", magic);
        }
        if header.version != 0 {
            let version = header.version;
            bail!("Unsupported PSF2 version: `{}`", version);
        }

        let mut glyphs = Vec::with_capacity(header.length as usize);
        let glyph_data_size = (header.length * header.glyph_size) as usize;
        let mut glyph_data = vec![0u8; glyph_data_size];
        file.read_exact(&mut glyph_data)?;

        for i in 0..header.length as usize {
            let start = i * header.glyph_size as usize;
            let end = start + header.glyph_size as usize;
            glyphs.push(glyph_data[start..end].to_vec());
        }

        let unicode_table = None;

        Ok(Self {
            header,
            glyphs,
            unicode_table,
        })
    }
}

pub enum PsfFont {
    V1(Psf1Font),
    V2(Psf2Font),
}

impl PsfFont {
    pub fn from_file(mut file: File) -> Result<Self> {
        let mut peek_bytes = [0u8; 4];
        let bytes_read = file.read(&mut peek_bytes)?;

        if bytes_read < 4 {
            bail!("File too small to be a PSF font");
        }

        file.seek(std::io::SeekFrom::Start(0))?;

        let possible_psf2_magic = u32::from_le_bytes(peek_bytes);
        if possible_psf2_magic == Psf2Font::MAGIC {
            return Ok(Self::V2(Psf2Font::from_file(file)?));
        }

        let possible_psf1_magic = u16::from_le_bytes([peek_bytes[0], peek_bytes[1]]);
        if Psf1Font::MAGICS.contains(&possible_psf1_magic) {
            return Ok(Self::V1(Psf1Font::from_file(file)?));
        }

        bail!(
            "File is neither PSF1 nor PSF2 format. Magic bytes: {:02x?}",
            &peek_bytes[0..4]
        )
    }
}
