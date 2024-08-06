use clap::{Parser, Subcommand};
use std::{fs, io, path::PathBuf};

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

impl Arg {
    fn get_path(&self) -> Result<PathBuf, io::Error> {
        let mut return_path = PathBuf::new();

        if let SubCommand::Init { gpg_id: _, path } = &self.subcommand {
            match path {
                Some(x) => {
                    return_path.push(x);
                    if !return_path.try_exists()? {
                        fs::create_dir_all(&return_path)?;
                    }
                }
                None => return_path.push("~/.local/share/rs-passstore"),
            }
        };

        Ok(return_path)
    }
}

fn main() {
    let args = Arg::parse();
    println!("{:?}", args.get_path());
}
