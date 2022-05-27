use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name="🦕 Stegosaurust", about="Hide text in images, using rust.")]
pub struct Opt {
    /// Decode a message from the image
    #[structopt(short,long)]
    pub decode: bool,
    
    /// Encode/decode with base64
    #[structopt(short,long)]
    pub base64: bool,

    /// Encrypt the text before encoding it with AES-128-CBC
    #[structopt(short,long)]
    pub key: Option<String>,

    /// Output file, stdout if not present
    #[structopt(short,long,parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Input file to encode, stdin if not present
    #[structopt(short,long,parse(from_os_str),conflicts_with="decode")]
    pub input: Option<PathBuf>,

    /// Input image
    #[structopt(parse(from_os_str))]
    pub image: PathBuf,

}

