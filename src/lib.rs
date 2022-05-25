use std::path::PathBuf;
use std::io::Write;

use structopt::StructOpt;
use anyhow::{Context,Result,bail};
use image::io::Reader as ImageReader;
use image::{guess_format,ImageFormat};


mod steganography;
use steganography::{Lsb,Steganography};

#[derive(StructOpt)]
#[structopt(name="stegosaurust", about="hide text in images, using rust.")]
pub struct Opt {
    #[structopt(long)]
    debug: bool,

    #[structopt(short,long)]
    decode: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

pub fn run(opt: Opt) -> Result<()> {
    let img = ImageReader::open(opt.input.clone())?.decode()?;
    let rgb8_img = img.into_rgb8();
    // TODO: this check is failing, do elsehow (file extension)
    let format = guess_format(&rgb8_img).with_context(|| format!("error processing {}",opt.input.to_str().unwrap()))?;
    if format == ImageFormat::Jpeg {
        bail!("Cannot use Jpeg for steganography");
    }

    let lsb = Lsb::new();

    if opt.decode {
        todo!("implement decoding of message from image");
    } else {
        let result = lsb.encode(&rgb8_img, "hello world").context("failed to encode message")?;
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