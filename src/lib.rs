use std::path::PathBuf;
use std::io::Write;

use structopt::StructOpt;
use image::io::Reader as ImageReader;
use image::{RgbImage};
use itertools::Itertools;

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
    let mut rgb8_img: RgbImage = img.into_rgb8();

    if opt.decode {
        todo!("implement decoding of message from image");
    } else {
        encode(&mut rgb8_img, opt)?
    }
    Ok(())    
}

fn encode(img: &mut RgbImage, opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: move to struct-opt
    let msg = "hello world".as_bytes();
    let mut binary_msg = String::with_capacity(msg.len()*7);
    // TODO: map this to enum, or better just parse to int (0,1)
    for byte in msg {
        binary_msg += &format!("{:b}", byte);
    }
    
    let mut ctr = 0;
    for chunk in &binary_msg.chars().chunks(3) {
        let x = ctr % img.width();
        let y = ctr / img.width();
        let pixel = img.get_pixel_mut(x, y);
        for (idx, bit) in chunk.enumerate() {
            match bit {
                '0' => pixel[idx] &= 0,
                '1' => pixel[idx] |= 1,
                _ => unreachable!("unreachable")
            }
        } 
        ctr+=1;
    }
    match opt.output {
        Some(path) => img.save(path)?,
        None => {
            let mut out = std::io::stdout();
            out.write_all(img.as_raw())?;
            out.flush()?;
        }
    }
    Ok(())
}