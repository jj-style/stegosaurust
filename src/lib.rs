use std::path::PathBuf;
use std::io::Write;

use structopt::StructOpt;
use image::io::Reader as ImageReader;
use image::{RgbImage};
use itertools::Itertools;

#[derive(StructOpt)]
#[structopt(name="stegosaurust", about="hide text in images, using rust.")]
pub struct Opt {
    #[structopt(short,long)]
    debug: bool,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

pub fn run(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageReader::open(opt.input)?.decode()?;
    let mut rgb8_img: RgbImage = img.into_rgb8();

    // TODO: move to struct-opt
    let msg = "hello world".as_bytes();
    let mut binary_msg = String::with_capacity(msg.len()*7);
    for byte in msg {
        binary_msg += &format!("{:b}", byte);
    }
    
    let mut ctr = 0;
    for chunk in &binary_msg.chars().chunks(3) {
        let x = ctr % rgb8_img.width();
        let y = ctr / rgb8_img.width();
        let pixel = rgb8_img.get_pixel_mut(x, y);
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
        Some(path) => rgb8_img.save(path)?,
        None => {
            let mut out = std::io::stdout();
            out.write_all(&rgb8_img.into_raw())?;
            out.flush()?;
        }
    }
    Ok(())
}