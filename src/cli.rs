use anyhow::{bail, Result};
use std::path::PathBuf;
use std::str::FromStr;
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

    /// Compress/decompress data
    #[structopt(short, long)]
    pub compress: bool,

    /// Encrypt the text before encoding it with AES-256-CBC
    #[structopt(short, long)]
    pub key: Option<String>,

    /// Check max message size that can be encoded with options given. Does not perform the encoding, acts like a dry-run
    #[structopt(short = "C", long)]
    pub check_max_length: bool,

    /// Method to use for encoding (lsb,rsb)
    #[structopt(short, long, default_value = "lsb")]
    pub method: StegMethod,

    /// Method for bit distribution (sequential, linear, random)
    #[structopt(long, default_value = "sequential")]
    pub distribution: BitDistribution,

    /// Seed for random significant bit encoding
    #[structopt(short, long, required_if("method", "rsb"))]
    pub seed: Option<String>,

    /// Maximum bit to possible modify (1-4)
    #[structopt(short = "N", long, required_if("method", "rsb"))]
    pub max_bit: Option<u8>,

    /// Seed for random bit distribution
    #[structopt(long, required_if("distribution", "random"))]
    pub distribution_seed: Option<String>,

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

/// Supported steganography encoding algorithms
#[derive(StructOpt, Debug)]
pub enum StegMethod {
    /// Least significant bit encoding
    ///
    /// With a binary message, each bit of the message is encoded
    /// into the least significant bit of each RGB byte of each pixel.
    LeastSignificantBit,
    /// Random significant bit encoding
    ///
    /// With a binary message, each bit of the message is encoded
    /// randomly into one of the `n` least significant bits of each RGB byte of each pixel.
    RandomSignificantBit,
}

impl FromStr for StegMethod {
    type Err = String;
    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method {
            "lsb" => Ok(Self::LeastSignificantBit),
            "rsb" => Ok(Self::RandomSignificantBit),
            other => Err(format!("unknown encoding method: {}", other)),
        }
    }
}

/// Supported bit encoding bit distribution methods
#[derive(StructOpt, Debug)]
pub enum BitDistribution {
    /// Encode bits sequentially into the image starting from top-left
    Sequential,
    /// Evenly space out the bits in the image so not all packed into top-left
    Linear,
    /// Based on a random-seed, encode each bit into a random pixel and random colour channel
    Random,
}

impl FromStr for BitDistribution {
    type Err = String;
    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method {
            "sequential" => Ok(Self::Sequential),
            "linear" => Ok(Self::Linear),
            "random" => Ok(Self::Random),
            other => Err(format!("unknown bit distribution {}", other)),
        }
    }
}

impl Default for BitDistribution {
    fn default() -> Self {
        BitDistribution::Linear
    }
}
