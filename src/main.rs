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

    let input_file = File::open(&args.input)?;
    let _input_font = PsfFont::from_file(input_file)?;

    Ok(())
}
