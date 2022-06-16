use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use atty::Stream;
use image::io::Reader as ImageReader;
use pretty_bytes::converter::convert;
use tabled::Table;

use crate::cli;
use crate::compress::{compress, decompress};
use crate::crypto;
use crate::steganography::{BitEncoder, DisguiseAssets, Lsb, Rsb, StegMethod, Steganography};

fn load_rgb8_img(path: &PathBuf) -> Result<image::RgbImage> {
    let img = ImageReader::open(path)
        .context(format!("opening {:?}", path))?
        .decode()?;
    Ok(img.into_rgb8())
}

/// Performs the steganography from the given command line options. Called from `main`.
pub fn run(opt: cli::Opt) -> Result<()> {
    match opt.cmd {
        cli::Command::Disguise(opts) => disguise(opts),
        cli::Command::Encode(opts) => encode(opts),
    }
}

/// perform an encoding
fn encode(opt: cli::Encode) -> Result<()> {
    let rgb8_img = load_rgb8_img(&opt.image)?;

    // create encoder
    let mut encoder: Box<dyn Steganography> = match opt.opts.method {
        StegMethod::LeastSignificantBit => {
            let lsb = Box::new(Lsb::new());
            Box::new(BitEncoder::new(lsb))
        }
        StegMethod::RandomSignificantBit => {
            let rsb = Box::new(Rsb::new(
                opt.opts.max_bit.unwrap(),
                &(opt.opts.seed.unwrap()),
            ));
            Box::new(BitEncoder::new(rsb))
        }
    };

    let max_msg_len = encoder.max_len(&rgb8_img);
    if opt.check_max_length {
        let table = Table::new(vec![
            ("Image", opt.image.to_str().unwrap()),
            ("Encoding Method", &format!("{:?}", opt.opts.method)),
            ("Max Message Length", &convert(max_msg_len as f64)),
        ])
        .with(tabled::Style::blank())
        .with(tabled::Disable::Row(..1))
        .with(tabled::Modify::new(tabled::object::Segment::all()).with(tabled::Alignment::left()))
        .to_string();
        println!("{}", table);
        return Ok(());
    }

    if opt.opts.decode {
        let mut result = encoder
            .decode(&rgb8_img)
            .context("failed to decode message from image")?;

        // perform transformations if necessary, decode then decrypt
        if opt.opts.base64 {
            result = base64::decode(result).context("failed to decode as base64")?;
        }

        if let Some(key) = opt.opts.key {
            result =
                crypto::decrypt(&result, key.as_bytes()).context("failed to decrypt message")?;
        }

        if opt.opts.compress {
            result = decompress(&result)?;
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
            print!("{}", result);
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

        if opt.opts.compress {
            message = compress(&message)?;
        }

        if let Some(key) = &opt.opts.key {
            message =
                crypto::encrypt(&message, key.as_bytes()).context("failed to encrypt message")?;
        }

        if opt.opts.base64 {
            message = base64::encode(&message).as_bytes().to_vec();
        }

        // check for message too long!
        if message.len() > max_msg_len {
            bail!(
                "Mesesage is too long, exceeds capacity that can fit in the image supplied. {} > {}
Try again using the compression flag --compress/-c, if not please use a larger image or less data",
                convert(message.len() as f64),
                convert(max_msg_len as f64)
            );
        }

        // encode
        let result = encoder
            .encode(&rgb8_img, &message)
            .context("failed to encode message")?;
        match opt.output {
            Some(path) => {
                println!("path is {:?}", &path);
                result.save(path)?;
            }
            None => {
                let mut out = std::io::stdout();
                out.write_all(result.as_raw())?;
                out.flush()?;
            }
        }
    }
    Ok(())
}

/// Disguise all files in directory by encoding them with assets embedded in the program
fn disguise(opt: cli::Disguise) -> Result<()> {
    if opt.opts.decode {
        for (_, entry) in std::fs::read_dir(&opt.dir)
            .context(format!("reading {:?}", opt.dir))?
            .enumerate()
        {
            let entry = entry?;
            if entry.path().is_file() {
                let entry = entry.path();
                let fname = entry.file_stem().unwrap();
                let fname = fname.to_str().unwrap();
                if let Ok(original_fname) = base64::decode(fname.as_bytes()) {
                    let original_fname = std::str::from_utf8(&original_fname).unwrap();
                    let mut new_path = entry.clone();
                    new_path.set_file_name(original_fname);
                    println!("decoding to {:?}", new_path);
                    encode(cli::Encode {
                        check_max_length: false,
                        opts: opt.opts.clone(),
                        input: None,
                        output: Some(new_path), // where to restore
                        image: entry.clone(),   // image to decode
                    })?;
                    std::fs::remove_file(entry)?;
                }
            }
        }
    } else {
        let assets = DisguiseAssets::iter()
            .map(|a| a.to_string())
            .collect::<Vec<String>>();
        let mut assets = assets.iter().cycle();
        for (_, entry) in std::fs::read_dir(&opt.dir)
            .context(format!("reading {:?}", opt.dir))?
            .enumerate()
        {
            let entry = entry?;
            if entry.path().is_file() {
                let mask = assets.next().unwrap();
                let mask = Path::new(mask);
                println!("encode {} with {:?}", entry.path().display(), mask);

                let mut new_fname: PathBuf = entry.path().clone();
                new_fname.set_file_name(base64::encode(
                    new_fname.file_name().unwrap().to_str().unwrap(),
                ));
                new_fname.set_extension("png");

                encode(cli::Encode {
                    check_max_length: false,
                    opts: opt.opts.clone(),
                    input: Some(entry.path()), // what to hide
                    output: Some(new_fname),   // where to hide
                    image: mask.to_owned(),    // image to hide in
                })?;
                std::fs::remove_file(entry.path())?;
            }
        }
    }
    Ok(())
}
