use crate::steganography::StegMethod;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "🦕 stegosaurust", about = "Hide text in images, using rust.")]
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    Encode(Encode),
    Disguise(Disguise),
}

#[derive(StructOpt)]
pub struct Encode {
    #[structopt(flatten)]
    pub opts: EncodeOpts,

    /// Check max message size that can be encoded with options given. Does not perform the encoding, acts like a dry-run
    #[structopt(short = "C", long)]
    pub check_max_length: bool,

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

#[derive(StructOpt)]
pub struct Disguise {
    #[structopt(flatten)]
    pub opts: EncodeOpts,

    /// Directory containing files to disguise
    #[structopt(parse(from_os_str))]
    pub dir: PathBuf,
}

#[derive(StructOpt, Clone)]
pub struct EncodeOpts {
    /// Enable debugging messages
    #[structopt(long)]
    pub debug: bool,

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

    /// Method to use for encoding (lsb,rsb)
    #[structopt(short, long, default_value = "lsb")]
    pub method: StegMethod,

    /// Seed for random significant bit encoding
    #[structopt(short, long, required_if("method", "rsb"))]
    pub seed: Option<String>,

    /// Maximum bit to possible modify (1-4)
    #[structopt(short = "N", long, required_if("method", "rsb"))]
    pub max_bit: Option<u8>,
}
