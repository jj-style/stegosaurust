use std::fs::File;
use std::io::{stdin, stdout, Read, Write};

use anyhow::{bail, Context, Result};
use atty::Stream;

use image::io::Reader as ImageReader;

use crate::cli;
use crate::crypto;
use crate::steganography::{BitEncoder, Lsb, Rsb, StegMethod, Steganography};
use pretty_bytes::converter::convert;
use tabled::Table;

/// Performs the steganography from the given command line options. Called from `main`.
pub fn run(opt: cli::Opt) -> Result<()> {
    let img = ImageReader::open(opt.image.clone())
        .context(format!("opening {}", opt.image.to_str().unwrap()))?
        .decode()?;
    let rgb8_img = img.into_rgb8();

    // create encoder
    let mut encoder: Box<dyn Steganography> = match opt.method {
        StegMethod::LeastSignificantBit => {
            let lsb = Box::new(Lsb::new());
            Box::new(BitEncoder::new(lsb))
        }
        StegMethod::RandomSignificantBit => {
            let rsb = Box::new(Rsb::new(opt.max_bit.unwrap(), &(opt.seed.unwrap())));
            Box::new(BitEncoder::new(rsb))
        }
    };
    let max_msg_len = encoder.max_len(&rgb8_img);
    if opt.check_max_length {
        let style = tabled::Style::blank();
        let table = Table::new(vec![
            ("Image", opt.image.to_str().unwrap()),
            ("Encoding Method", &format!("{:?}", opt.method)),
            ("Max Message Length", &convert(max_msg_len as f64)),
        ])
        .with(style)
        .with(tabled::Disable::Row(..1))
        .with(tabled::Modify::new(tabled::object::Segment::all()).with(tabled::Alignment::left()))
        .to_string();
        println!("{}", table);
        return Ok(());
    }

    if opt.decode {
        let mut result = encoder
            .decode(&rgb8_img)
            .context("failed to decode message from image")?;

        // perform transformations if necessary, decode then decrypt
        if opt.base64 {
            result = base64::decode(result).context("failed to decode as base64")?;
        }

        if let Some(key) = opt.key {
            result =
                crypto::decrypt(&result, key.as_bytes()).context("failed to decrypt message")?;
        }

        if let Some(path) = opt.output {
            let mut f = File::create(&path)
                .context(format!("failed to create file: {}", path.to_str().unwrap()))?;
            f.write_all(&result)
                .context("failed to write message to file")?;
        } else {
            let result = match String::from_utf8(result.clone()) {
                Ok(s) => s,
                Err(_) => unsafe { String::from_utf8_unchecked(result) },
            };
            println!("{}", result);
        }
    } else {
        // read message to encode to image from file/stdin
        let mut message = match &opt.input {
            Some(path) => {
                let mut file = File::open(path)
                    .context(format!("failed to read {}", path.to_str().unwrap()))?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                buffer
            }
            None => {
                let mut buffer = Vec::new();
                if atty::is(Stream::Stdin) {
                    print!("Enter message to encode: ");
                    let _ = stdout().flush();
                    let mut str_buf = String::new();
                    stdin().read_line(&mut str_buf)?;
                    buffer = str_buf.into_bytes();
                } else {
                    stdin().read_to_end(&mut buffer)?;
                }
                buffer
            }
        };

        // perform transformations if necessary, encrypt then encode
        if let Some(key) = &opt.key {
            message =
                crypto::encrypt(&message, key.as_bytes()).context("failed to encrypt message")?;
        }

        if opt.base64 {
            message = base64::encode(&message).as_bytes().to_vec();
        }

        // check for message too long!
        if message.len() > max_msg_len {
            bail!("Mesesage is too long, exceeds capacity that can fit in the image supplied. {} > {}", convert(message.len() as f64), convert(max_msg_len as f64));
        }

        // encode
        let result = encoder
            .encode(&rgb8_img, &message)
            .context("failed to encode message")?;
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
