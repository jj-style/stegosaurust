use std::path::PathBuf;
use std::io::Write;

use structopt::StructOpt;
use image::io::Reader as ImageReader;

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

pub fn run(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageReader::open(opt.input.clone())?.decode()?;

    let lsb = Lsb::new(img.into_rgb8());

    if opt.decode {
        todo!("implement decoding of message from image");
    } else {
        let result = lsb.encode("hello world")?;
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