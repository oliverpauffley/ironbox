use std::{collections::HashMap, env, io::Write, os, path::PathBuf, process::Command};

use aes_gcm_siv::{Aes256GcmSiv, KeyInit};
use clap::{arg, Parser, Subcommand};
use flate2::{write::GzEncoder, Compression, GzBuilder};
use home::home_dir;

use crate::key_ring::{Key, KeyEntry, KeyID, KeyRing};

mod key_ring;

/// Encrypt and decrypt secrets with git
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate the appropriate git config for using ironbox
    GitConfig,
    /// Decrypt files at a given path
    Decrypt {
        /// path of the keyring to decrypt
        #[arg(short, long)]
        keyring: Option<PathBuf>,

        /// recursively decrypt all files under a given folder
        #[arg(short, long)]
        recursive: bool,

        path: PathBuf,
    },
    /// Generate a new key for secrets
    GenKey {
        /// path of the keyring to attach the key
        #[arg(short, long)]
        keyring: Option<PathBuf>,

        /// descriptive name for the new key
        description: String,
    },

    Clean,
    Smudge,
    Diff,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut user_home = get_user_home()?;

    if let Some(command) = args.command {
        match command {
            Commands::GitConfig => return gen_git_config(),
            Commands::Decrypt {
                keyring,
                recursive,
                path,
            } => todo!(),
            Commands::GenKey {
                keyring,
                description: key_description,
            } => {
                let path = get_key_ring_path(keyring)?;
                let mut ring = KeyRing::load(path.clone())?;
                ring.gen_key(key_description).unwrap();
                ring.save(path).unwrap();
            }
            Commands::Clean => todo!(),
            Commands::Smudge => todo!(),
            Commands::Diff => todo!(),
        }
    }

    Ok(())
}

fn encrypt(key: Key, input: &str) -> anyhow::Result<&str> {
    let bytes = compress(input.as_bytes())?;
    let cipher = Aes256GcmSiv::new(key);
    let out = siv
}

fn compress(b: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::fast());
    e.write_all(b)?;
    Ok(e.finish()?)
}

fn gen_git_config() -> anyhow::Result<()> {
    let mut command = Command::new("git");
    command.args([
        "config",
        "--global",
        "--replace-all",
        "filter.ironbox.clean",
        "'ironbox clean %f",
    ]);
    command.status()?;
    let mut command = Command::new("git");
    command.args([
        "config",
        "--global",
        "--replace-all",
        "filter.ironbox.smudge",
        "'ironbox smudge %f",
    ]);
    command.status()?;
    let mut command = Command::new("git");
    command.args([
        "config",
        "--global",
        "--replace-all",
        "filter.ironbox.required",
        "true",
    ]);
    command.status()?;
    let mut command = Command::new("git");
    command.args([
        "config",
        "--global",
        "--replace-all",
        "diff.ironbox.textconv",
        "'ironbox diff",
    ]);
    command.status()?;

    Ok(())
}

fn get_key_ring_path(keyring: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    Ok(keyring.unwrap_or({
        let mut user_home = get_user_home()?;
        user_home.push(".strongbox_keyring");
        user_home
    }))
}

fn get_user_home() -> anyhow::Result<PathBuf> {
    // check for the env var for the home to use
    if let Ok(home) = env::var("STRONGBOX_HOME") {
        return Ok(home.into());
    };

    if let Some(home) = home_dir() {
        return Ok(home);
    };

    Err(anyhow::anyhow!(
        "Failed to get user home directory, or find $STRONGBOX_HOME"
    ))
}
