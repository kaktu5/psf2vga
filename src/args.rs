use std::path::PathBuf;

use clap::Parser;

/// Convert a PC Screen Font to VGA text mode font (Code Page 437)
#[derive(Parser)]
pub struct Args {
    /// Input PSF font file
    #[arg(env)]
    pub input_path: PathBuf,

    /// Output VGA font file
    #[arg(env)]
    pub output_path: Option<PathBuf>,

    /// Generate preview images
    #[arg(long)]
    pub preview: bool,

    /// Increase logging verbosity
    #[arg(short, long, env)]
    pub verbose: bool,
}
