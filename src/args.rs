use std::path::PathBuf;

use clap::Parser;

/// Convert a PC Screen Font to VGA text mode font (Code Page 437)
#[derive(Parser)]
pub struct Args {
    /// Input PSF font file
    pub input_path: PathBuf,

    /// Output VGA font file
    pub output_path: Option<PathBuf>,

    /// Generate preview images
    #[arg(long)]
    pub preview: bool,

    /// Decrease logging verbosity
    #[arg(short, long)]
    pub quiet: bool,
}
