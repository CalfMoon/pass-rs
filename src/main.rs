use std::path::PathBuf;

use clap::Parser;

/// A simple password manager written in rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Add a new password
    #[arg(short, long)]
    new: Option<String>,

    /// Add a new password
    #[arg(short, long)]
    read: Option<String>,

    /// Inatilize a password diretory
    #[arg(long, value_name = "Path")]
    init: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
}
