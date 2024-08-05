use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A simple password manager written in rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arg {
    #[clap(flatten)]
    options: Options,

    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Parser, Debug)]
struct Options {}

#[derive(Debug, Subcommand, Clone)]
enum SubCommand {
    /// add a new password
    New { name: String },

    /// read a existing password
    Read { name: String },

    /// inatilize a new password store
    Init {
        /// pgp key to use
        gpg_id: String,

        /// where the password-store is inisialized
        #[arg(long, short)]
        path: Option<PathBuf>,
    },
}

fn main() {
    let args = Arg::parse();
}
