use crate::steganography::StegMethod;
use anyhow::{bail, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "🦕 stegosaurust", about = "Hide text in images, using rust.")]
pub struct Opt {
    /// Decode a message from the image
    #[structopt(short, long)]
    pub decode: bool,

    /// Encode/decode with base64
    #[structopt(short, long)]
    pub base64: bool,

    /// Encrypt the text before encoding it with AES-256-CBC
    #[structopt(short, long)]
    pub key: Option<String>,

    /// Check max message size that can be encoded with options given. Does not perform the encoding, acts like a dry-run
    #[structopt(short, long)]
    pub check_max_length: bool,

    /// Method to use for encoding (lsb,rsb)
    #[structopt(short, long, default_value = "lsb")]
    pub method: StegMethod,

    /// Seed for random significant bit encoding
    #[structopt(short, long, required_if("method", "rsb"))]
    pub seed: Option<String>,

    /// Maximum bit to possible modify (1-4)
    #[structopt(short = "N", long, required_if("method", "rsb"))]
    pub max_bit: Option<u8>,

    /// Output file, stdout if not present
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Input file to encode, stdin if not present
    #[structopt(short, long, parse(from_os_str), conflicts_with = "decode")]
    pub input: Option<PathBuf>,

    /// Input image
    #[structopt(parse(from_os_str))]
    pub image: PathBuf,
}

impl Opt {
    pub fn validate(&self) -> Result<()> {
        if let Some(n) = self.max_bit {
            if !(1..=4).contains(&n) {
                bail!(format!("max-bit must be between 1-4. Got {}", n))
            }
        }
        Ok(())
    }
}
