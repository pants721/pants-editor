use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    /// File to edit
    pub file: Option<String>,
    /// Use this config 
    #[arg(short = 'u')]
    pub config: Option<PathBuf>,
}

