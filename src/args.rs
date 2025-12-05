use std::path::PathBuf;

use clap::Parser;

/// Convert a PSF console font to VGA text mode font (CP437)
#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// Input PSF font file
    pub input: PathBuf,

    /// Output VGA font file
    pub output: PathBuf,

    /// Use verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
