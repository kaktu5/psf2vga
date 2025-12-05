mod args;
mod psf;

use std::fs::File;

use args::Args;
use clap::Parser as _;
use color_eyre::Result;
use psf::PsfFont;

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let input_file = File::open(&args.input_path)?;
    let font = PsfFont::from_file(input_file)?;

    println!("Font:");
    println!("  Glyph size: {}x{}", font.glyph_size.0, font.glyph_size.1);
    println!("  Glyphs: {}", font.glyphs.len());
    if let Some(unicode_table) = font.unicode_table {
        println!("  Unicode table length: {}", unicode_table.len());
    }

    Ok(())
}
