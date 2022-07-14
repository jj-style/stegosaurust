use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "🦕 stegosaurust", about = "Hide text in images, using rust.")]
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(name = "enc", about = "encode files using steganography")]
    Encode(Encode),
    #[structopt(
        name = "disguise",
        about = "mask all files in a directory using steganography"
    )]
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

    /// Method to use for encoding [default=lsb]
    #[structopt(short, long, possible_values=&StegMethod::variants())]
    pub method: Option<StegMethod>,

    /// Method for bit distribution [default=sequential] [possible values: sequential, linear (linear-N when decoding)]
    #[structopt(long)]
    pub distribution: Option<BitDistribution>,

    /// Seed for random significant bit encoding
    #[structopt(short, long, required_if("method", "rsb"))]
    pub seed: Option<String>,

    /// Maximum bit to possible modify
    #[structopt(short = "N", long, required_if("method", "rsb"), possible_values=&["1","2","3","4"])]
    pub max_bit: Option<u8>,
}

/// Supported steganography encoding algorithms
#[derive(StructOpt, Debug, Clone, Copy)]
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

impl Default for StegMethod {
    fn default() -> Self {
        StegMethod::LeastSignificantBit
    }
}

impl StegMethod {
    fn variants() -> [&'static str; 2] {
        ["lsb", "rsb"]
    }
}

/// Supported bit encoding bit distribution methods
#[derive(StructOpt, Debug, Clone)]
pub enum BitDistribution {
    /// Encode bits sequentially into the image starting from top-left
    Sequential,
    /// Evenly space out the bits in the image so not all packed into top-left
    Linear { length: usize },
}

impl FromStr for BitDistribution {
    type Err = String;
    fn from_str(method: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = method.split('-').collect();
        let method = if parts.len() <= 1 { method } else { parts[0] };
        match method {
            "sequential" => Ok(Self::Sequential),
            "linear" => {
                let length = *(parts.get(1).unwrap_or(&"0"));
                let length = length.parse::<usize>().unwrap_or_else(|err| {
                    eprintln!(
                        "error parsing message length in linear bit distribution: {}",
                        err
                    );
                    std::process::exit(1);
                });
                Ok(Self::Linear { length })
            }
            other => Err(format!("unknown bit distribution {}", other)),
        }
    }
}

impl Default for BitDistribution {
    fn default() -> Self {
        BitDistribution::Sequential
    }
}
