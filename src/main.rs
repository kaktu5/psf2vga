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
use color_eyre::{Result, eyre::Context as _};
use image::codecs::webp::WebPEncoder;
use preview::FontPreview;
use psf::PsfFont;
use tap::{Pipe as _, Tap as _};
use vga::VgaFont;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let psf_font = File::open(&args.input_path)
        .wrap_err_with(|| format!("Failed to open input file: `{}`", args.input_path.display()))?
        .pipe(PsfFont::from_file)
        .wrap_err_with(|| format!("Failed to parse PSF from: `{}`", args.input_path.display()))?;
    if !args.quiet {
        println!("Loaded PSF from: `{}`", args.input_path.display());
    }
    if args.preview {
        save_preview(&psf_font, &args.input_path, args.quiet)?;
    }

    let vga_font = VgaFont::try_from(psf_font).wrap_err("Failed to convert PSF to VGA font")?;
    if !args.quiet {
        println!("Converted PSF to VGA font");
    }

    let output_path = determine_output_path(&args.input_path, args.output_path, vga_font.height);
    save_vga_font(&vga_font, &output_path, args.quiet)?;
    if args.preview {
        save_preview(&vga_font, &output_path, args.quiet)?;
    }

    Ok(())
}

fn save_preview(font: &impl FontPreview, base_path: &Path, quiet: bool) -> Result<()> {
    let preview_path = base_path
        .as_os_str()
        .to_os_string()
        .tap_mut(|s| s.push(".webp"))
        .pipe(PathBuf::from);

    let img = font.preview();

    let file = File::create(&preview_path).with_context(|| {
        format!(
            "Failed to create preview file: `{}`",
            preview_path.display()
        )
    })?;

    let encoder = WebPEncoder::new_lossless(file);

    img.write_with_encoder(encoder).wrap_err_with(|| {
        format!(
            "Failed to encode preview image as WebP: `{}`",
            preview_path.display()
        )
    })?;

    if !quiet {
        println!("Preview saved to: `{}`", preview_path.display());
    }

    Ok(())
}

fn determine_output_path(input_path: &Path, output_path: Option<PathBuf>, height: u8) -> PathBuf {
    output_path.unwrap_or_else(|| {
        let prefix = input_path
            .file_prefix()
            .unwrap_or_default()
            .to_string_lossy();
        input_path.with_file_name(format!("{prefix}.f{height}"))
    })
}

fn save_vga_font(vga: &VgaFont, path: &Path, quiet: bool) -> Result<()> {
    let data: Vec<u8> = vga
        .glyphs
        .iter()
        .flat_map(|glyph| &glyph[..vga.height as usize])
        .copied()
        .collect();

    File::create(path)
        .wrap_err_with(|| format!("Failed to create VGA font file: `{}`", path.display()))?
        .write_all(&data)
        .wrap_err_with(|| format!("Failed to write VGA font data to: `{}`", path.display()))?;

    if !quiet {
        println!("VGA font saved to: `{}`", path.display());
    }

    Ok(())
}
