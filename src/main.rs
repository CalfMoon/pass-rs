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
    process::{Command, Stdio},
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
    Read {
        name: String,

        /// copy password into clipboard
        #[arg(long, short)]
        copy: bool,
    },

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
                None => {
                    return_path = PathBuf::from(if let Ok(x) = env::var("XDG_DATA_HOME") {
                        x
                    } else {
                        "~/.local/share/".to_string()
                    });
                    return_path.push("rs-passstore");
                }
            }
        }
        Ok(return_path)
    }

    fn get_gpg(&self) -> Result<String, io::Error> {
        if let SubCommand::Init { gpg_id, path: _ } = &self {
            let output = String::from_utf8(
                Command::new("gpg")
                    .args(["--list-secret-keys", "--keyid-format", "LONG", gpg_id])
                    .output()?
                    .stdout,
            )
            .unwrap();

            let regex = Regex::new(r".*\/(?<id>[A-Z0-9]+)[0-9- ]{12}\[E\]").unwrap();
            if let Some(key_id) = &regex.captures(&output) {
                return Ok(key_id["id"].to_string());
            } else {
                return Ok(gpg_id.clone());
            };
        }
        unreachable!()
    }

    fn write_config(&self, data: Config) -> Result<(), io::Error> {
        let mut config_directory = PathBuf::from(if let Ok(x) = env::var("XDG_CONFIG_HOME") {
            x
        } else {
            "~/.config/".to_string()
        });
        config_directory.push("pass-rs");

        fs::create_dir_all(&config_directory)?;
        let mut file = fs::File::create(config_directory.join("config.toml"))?;

        let data_string = toml::to_string(&data).unwrap();
        file.write(data_string.as_bytes())?;
        Ok(())
    }

    fn read_config(&self) -> Result<Config, Box<dyn Error>> {
        let config_directory = PathBuf::from(if let Ok(x) = env::var("XDG_CONFIG_HOME") {
            x
        } else {
            "~/.config/".to_string()
        });
        let data_string = fs::read_to_string(config_directory.join("pass-rs/config.toml"))?;
        let data: Config = toml::from_str(&data_string)?;

        Ok(data)
    }

    fn get_password(&self) -> Result<String, io::Error> {
        let mut password1 = String::new();
        let mut password2 = String::new();
        if let SubCommand::New { name } = &self {
            println!("Enter password for {}: ", name);
            io::stdin().read_line(&mut password1)?;

            println!("Retype password for {}: ", name);
            io::stdin().read_line(&mut password2)?;

            if password1 != password2 {
                eprintln!("The passwords don't match!");
                std::process::exit(-1);
            }
        }
        password1.pop();
        return Ok(password1);
    }

    fn write_password(
        &self,
        password: String,
        path: String,
        gpg_id: String,
    ) -> Result<(), io::Error> {
        let cmd_echo = Command::new("echo")
            .arg(password)
            .stdout(Stdio::piped())
            .spawn()?;

        Command::new("gpg")
            .args(["-e", "-r", &gpg_id, "-o", &path])
            .stdin(cmd_echo.stdout.unwrap())
            .spawn()?;

        return Ok(());
    }

    fn read_password(&self, path: String) -> Result<String, io::Error> {
        let output = Command::new("gpg").args(["-d", &path]).output()?.stdout;

        return Ok(String::from_utf8(output).unwrap());
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    store_path: PathBuf,
    gpg_id: String,
}

fn main() {
    let args = Arg::parse();
    let subcommand = args.subcommand;
    match &subcommand {
        SubCommand::Init { gpg_id: _, path: _ } => {
            let store_path = subcommand.get_store_path().unwrap();
            let gpg_id = subcommand.get_gpg().unwrap();
            subcommand
                .write_config(Config { store_path, gpg_id })
                .unwrap();
        }

        SubCommand::New { name } => {
            let password = subcommand.get_password().unwrap();
            let Config { store_path, gpg_id } = subcommand.read_config().unwrap();
            subcommand
                .write_password(
                    password,
                    String::from(store_path.join(name.clone() + ".gpg").to_string_lossy()),
                    gpg_id,
                )
                .unwrap()
        }

        SubCommand::Read { name, copy } => {
            let Config {
                store_path,
                gpg_id: _,
            } = subcommand.read_config().unwrap();

            let password = subcommand
                .read_password(String::from(
                    store_path.join(name.clone() + ".gpg").to_string_lossy(),
                ))
                .unwrap();

            if *copy {
                use arboard::Clipboard;
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(password).unwrap();
            } else {
                println!("{password}");
            }
        }
    }
}
