use std::path::PathBuf;
use std::io::{Write,stdin,stdout};

use structopt::StructOpt;
use anyhow::{Context,Result,bail};
use image::io::Reader as ImageReader;
use image::{ImageFormat};

mod steganography;
use steganography::{Lsb,Steganography};

#[derive(StructOpt)]
#[structopt(name="🦕 Stegosaurust", about="hide text in images, using rust.")]
pub struct Opt {
    #[structopt(short,long)]
    decode: bool,
    
    /// Encode/decode with base64
    #[structopt(short,long)]
    _base64: bool,

    /// Encrypt the text before encoding it with AES-256-CBC
    #[structopt(short,long)]
    _key: Option<String>,

    /// Output file, stdout if not present
    #[structopt(short,long,parse(from_os_str))]
    output: Option<PathBuf>,

    /// Input file to encode
    #[structopt(short,long,parse(from_os_str),conflicts_with="decode")]
    input: Option<PathBuf>,

    /// Input image
    #[structopt(parse(from_os_str))]
    image: PathBuf,

}

pub fn run(opt: Opt) -> Result<()> {
    let img = ImageReader::open(opt.image.clone()).context(format!("opening {}", opt.image.to_str().unwrap()))?.decode()?;
    let rgb8_img = img.into_rgb8();
    match ImageFormat::from_path(&opt.image).with_context(|| format!("error processing {}",opt.image.to_str().unwrap()))? {
        ImageFormat::Jpeg => bail!("Cannot use Jpeg for steganography"),
        _ => {}
    }

    let lsb = Lsb::new();

    if opt.decode {
        let result = lsb.decode(&rgb8_img).context("failed to decode message from image")?;
        if let Some(path) = opt.output {
            todo!("{}", format!("write decoded message to path {}", path.to_str().unwrap()));
        } else {
            println!("{}", String::from_utf8(result).unwrap());
        }

    } else {
        let message = match &opt.input {
            Some(path) => {
                std::fs::read_to_string(path)?
            },
            None => {
                let mut buf = String::new();
                print!("Enter message to encode: ");
                let _ = stdout().flush();
                stdin().read_line(&mut buf)?;
                buf
            }
        };
        let result = lsb.encode(&rgb8_img, message.as_bytes()).context("failed to encode message")?;
        match opt.output {
            Some(path) => result.save(path)?,
            None => {
                let mut out = std::io::stdout();
                out.write_all(result.as_raw())?;
                out.flush()?;
            }
        }
    }
    Ok(())    
}