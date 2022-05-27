use std::io::{Write,Read,stdin,stdout};
use std::fs::File;

use anyhow::{Context,Result,bail};
use atty::Stream;
use image::io::Reader as ImageReader;
use image::{ImageFormat};
use base64;

use crate::cli;
use crate::steganography::{Lsb,Steganography,END};
use crate::crypto;

pub fn run(opt: cli::Opt) -> Result<()> {
    let img = ImageReader::open(opt.image.clone()).context(format!("opening {}", opt.image.to_str().unwrap()))?.decode()?;
    let rgb8_img = img.into_rgb8();
    match ImageFormat::from_path(&opt.image).with_context(|| format!("error processing {}",opt.image.to_str().unwrap()))? {
        ImageFormat::Jpeg => bail!("Cannot use Jpeg for steganography"),
        _ => {}
    }

    let lsb = Lsb::new();

    if opt.decode {
        let mut result = lsb.decode(&rgb8_img).context("failed to decode message from image")?;
        
        // perform transformations if necessary, decode then decrypt
        if opt.base64 {
            result = base64::decode(result).context("failed to decode as base64")?;
        }

        if let Some(key) = opt.key {
            result = crypto::decrypt(&result, key.as_bytes()).context("failed to decrypt message")?; 
        }
 
        if let Some(path) = opt.output {
            let mut f = File::create(&path).context(format!("failed to create file: {}", path.to_str().unwrap()))?;
            f.write_all(&result).context("failed to write message to file")?;
        } else {
            let result = match String::from_utf8(result.clone()){
                Ok(s) => s,
                Err(_) => unsafe {
                    String::from_utf8_unchecked(result)
                }
            };
            println!("{}", result);
        }

    } else {
        // read message to encode to image from file/stdin
        let mut message = match &opt.input {
            Some(path) => {
                let mut file = File::open(path).context(format!("failed to read {}", path.to_str().unwrap()))?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                buffer
            },
            None => {
                let mut buf = String::new();
                if atty::is(Stream::Stdin) {
                    print!("Enter message to encode: ");
                    let _ = stdout().flush();
                }
                stdin().read_line(&mut buf)?;
                buf.as_bytes().to_vec()
            }
        };
         
        // perform transformations if necessary, encrypt then encode
        if let Some(key) = opt.key {
           message = crypto::encrypt(&message, key.as_bytes()).context("failed to encrypt message")?; 
        }
        
        if opt.base64 {
            message = base64::encode(&message).as_bytes().to_vec();
        }
        
        // check for message too long!
        let max_msg_len = ((rgb8_img.width()*rgb8_img.height()*3) as usize - (END.len()*8)) / 8;
        if message.len() > max_msg_len {
            bail!("Mesesage is too long, exceeds capacity that can fit in the image supplied.");
        }

        // encode
        let result = lsb.encode(&rgb8_img, &message).context("failed to encode message")?;
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