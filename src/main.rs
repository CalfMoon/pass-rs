use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use toml;

use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
    process,
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

impl SubCommand {
    fn get_store_path(&self) -> Result<PathBuf, io::Error> {
        let mut return_path = PathBuf::new();

        if let SubCommand::Init { gpg_id: _, path } = &self {
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

    fn get_gpg(&self) -> Result<String, io::Error> {
        if let SubCommand::Init { gpg_id, path: _ } = &self {
            let output = String::from_utf8(
                process::Command::new("gpg")
                    .args(["--list-secret-keys", "--keyid-format", "LONG", gpg_id])
                    .output()?
                    .stdout,
            )
            .unwrap();

            let regex = Regex::new(r".*\/(?<id>[A-Z0-9]+)[0-9- ]{12}\[E\]").unwrap();
            if let Some(key_id) = &regex.captures(&output) {
                println!("{}", key_id["id"].to_string());
                return Ok(key_id["id"].to_string());
            } else {
                return Ok(gpg_id.clone());
            };
        }
        unreachable!()
    }

    fn write_config(&self, data: Config) -> Result<(), Box<dyn Error>> {
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

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    store_path: PathBuf,
}

fn main() {
    let args = Arg::parse();
    let subcommand = args.subcommand;
    match subcommand {
        SubCommand::Init { gpg_id: _, path: _ } => {
            let store_path = subcommand.get_store_path().unwrap();
            subcommand.write_config(Config { store_path }).unwrap();
        }
        SubCommand::New { name: _ } => {}
        SubCommand::Read { name: _ } => {}
    }
}
