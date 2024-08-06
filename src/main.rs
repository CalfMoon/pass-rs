use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use toml;

use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
};

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
    fn get_store_path(&self) -> Result<PathBuf, io::Error> {
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

    fn write_to_config(&self, data: Config) -> Result<(), Box<dyn Error>> {
        let mut config_directory = PathBuf::from(if let Ok(x) = env::var("XDG_CONFIG_HOME") {
            x
        } else {
            "~/.config/".to_string()
        });
        config_directory.push("pass-rs");

        fs::create_dir_all(&config_directory)?;
        let mut file = fs::File::create(config_directory.join("config.json"))?;

        let dat = toml::to_string(&data)?;

        file.write(dat.as_bytes())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    store_path: PathBuf,
}

fn main() {
    let args = Arg::parse();
    match args.subcommand {
        SubCommand::Init { gpg_id: _, path: _ } => {
            let store_path = args.get_store_path().unwrap();
            args.write_to_config(Config { store_path }).unwrap();
        }
        SubCommand::New { name: _ } => {}
        SubCommand::Read { name: _ } => {}
    }
}
