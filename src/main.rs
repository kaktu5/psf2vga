mod args;
mod cp437;
mod preview;
mod psf;
mod vga;

use std::{
    fs::File,
    io::Write as _,
    path::{Path, PathBuf},
};

use args::Args;
use clap::Parser as _;
use color_eyre::Result;
use image::codecs::webp::WebPEncoder;
use preview::FontPreview;
use psf::PsfFont;
use vga::VgaFont;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let psf_font = PsfFont::from_file(File::open(&args.input_path)?)?;

    if args.preview {
        save_preview(&psf_font, &args.input_path, args.verbose)?;
    }

    let vga_font = VgaFont::try_from(psf_font)?;

    let output_path = args
        .output_path
        .unwrap_or_else(|| generate_output_path(&args.input_path, vga_font.height));

    save_vga_font(&vga_font, &output_path, args.verbose)?;

    if args.preview {
        save_preview(&vga_font, &output_path, args.verbose)?;
    }

    Ok(())
}

fn generate_output_path(input_path: &Path, height: u8) -> PathBuf {
    let stem = input_path.file_stem().unwrap_or_default();
    let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
    parent.join(format!("{}.f{}", stem.to_string_lossy(), height))
}

fn save_vga_font(vga: &VgaFont, path: &Path, verbose: bool) -> Result<()> {
    let mut file = File::create(path)?;

    for glyph in &vga.glyphs {
        file.write_all(&glyph[..vga.height as usize])?;
    }

    if verbose {
        println!("VGA font saved to: {}", path.display());
    }

    Ok(())
}

fn save_preview(font: &impl FontPreview, base_path: &Path, verbose: bool) -> Result<()> {
    let mut preview_path = base_path.to_path_buf();
    let current_extension = preview_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    preview_path.set_extension(format!("{current_extension}.webp"));

    let img = font.preview();

    let file = File::create(&preview_path)?;
    let encoder = WebPEncoder::new_lossless(file);
    img.write_with_encoder(encoder)?;

    if verbose {
        println!("Preview saved to: {}", preview_path.display());
    }

    Ok(())
}
