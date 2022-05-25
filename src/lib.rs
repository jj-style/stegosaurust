use std::path::PathBuf;
use std::io::Write;

use structopt::StructOpt;
use anyhow::{Context,Result,bail};
use image::io::Reader as ImageReader;
use image::{ImageFormat};


mod steganography;
use steganography::{Lsb,Steganography};

#[derive(StructOpt)]
#[structopt(name="stegosaurust", about="hide text in images, using rust.")]
pub struct Opt {
    // #[structopt(long)]
    // debug: bool,

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
    let img = ImageReader::open(opt.input.clone()).context(format!("opening {}", opt.input.to_str().unwrap()))?.decode()?;
    let rgb8_img = img.into_rgb8();
    match ImageFormat::from_path(&opt.input).with_context(|| format!("error processing {}",opt.input.to_str().unwrap()))? {
        ImageFormat::Jpeg => bail!("Cannot use Jpeg for steganography"),
        _ => {}
    }

    let lsb = Lsb::new();

    if opt.decode {
        let result = lsb.decode(&rgb8_img).context("failed to decode message from image")?;
        if let Some(path) = opt.output {
            todo!("{}", format!("write decoded message to path {}", path.to_str().unwrap()));
        } else {
            println!("{}", std::str::from_utf8(result)?);
        }

    } else {
        let result = lsb.encode(&rgb8_img, b"hello world").context("failed to encode message")?;
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