use std::path::PathBuf;

use clap::Parser;

/// Convert a PSF console font to VGA text mode font (CP437)
#[derive(Parser)]
pub struct Args {
    /// Input PSF font file
    #[arg(env)]
    pub input_path: PathBuf,

    /// Output VGA font file
    #[arg(env)]
    pub output_path: PathBuf,

    /// Generate a preview image of the input font
    #[arg(long)]
    pub input_preview: bool,

    /// Generate a preview image of the output font
    #[arg(long)]
    pub output_preview: bool,

    /// Use verbose output
    #[arg(short, long, env)]
    pub verbose: bool,
}
