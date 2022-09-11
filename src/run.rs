use std::fs::{DirEntry, File};
use std::io::{stdin, stdout, Cursor, Read, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use atty::Stream;
use image::io::Reader as ImageReader;
use log::{debug, error, warn};
use pretty_bytes::converter::convert;
use tabled::Table;

use crate::cli;
use crate::compress::{compress, decompress};
use crate::crypto;
use crate::steganography::encoder_from_opts;
use crate::StegError;

use crate::image_api::{self, ImageApi};

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
        cli::Command::Encode(opts) => {
            let rgb8_img = load_rgb8_img(&opts.image)?;
            encode(opts, rgb8_img)
        }
    }
}

/// perform an encoding
fn encode(opt: cli::Encode, mask: image::RgbImage) -> Result<()> {
    // let rgb8_img = load_rgb8_img(&opt.image)?;

    let steg_method = opt.opts.method.unwrap_or_default();

    // create encoder
    let mut encoder = encoder_from_opts(opt.opts.clone());

    let max_msg_len = encoder.max_len(&mask);
    if opt.check_max_length {
        let table = Table::new(vec![
            ("Image", opt.image.to_str().unwrap()),
            ("Encoding Method", &format!("{:?}", steg_method)),
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
            .decode(&mask)
            .context("failed to decode message from image")?;

        // perform transformations if necessary, decode then decrypt
        if opt.opts.base64 {
            result = base64::decode(result)?;
        }

        if let Some(key) = opt.opts.key {
            result = crypto::decrypt(&result, key.as_bytes()).map_err(StegError::Crypto)?;
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
            message = crypto::encrypt(&message, key.as_bytes()).map_err(StegError::Crypto)?;
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
            .encode(&mask, &message)
            .context("failed to encode message")?;
        match opt.output {
            Some(path) => {
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
        for (_, dirent) in std::fs::read_dir(&opt.dir)
            .context(format!("reading {:?}", opt.dir))?
            .into_iter()
            .filter(|r| r.is_ok())
            // SAFETY: since we only have the Ok variants from above `filter`
            .map(|r| r.unwrap())
            .filter(is_not_hidden)
            .enumerate()
        {
            if dirent.path().is_file() {
                let path = dirent.path();
                let fname = path.file_stem().unwrap();
                let fname = fname.to_str().unwrap();
                let original_fname = match base64::decode(fname.as_bytes()) {
                    Ok(res) => res,
                    Err(err) => {
                        warn!(
                            "error decoding original filename from {:?}: {:?}",
                            path, err
                        );
                        continue;
                    }
                };
                let original_fname = match std::str::from_utf8(&original_fname) {
                    Ok(res) => res,
                    Err(err) => {
                        warn!(
                            "error deriving original filename from {:?}: {:?}",
                            path, err
                        );
                        continue;
                    }
                };
                let mut new_path = path.clone();
                new_path.set_file_name(original_fname);

                let mask = load_rgb8_img(&path)?;

                debug!("decoding {} ==> {}", path.display(), new_path.display());

                match encode(
                    cli::Encode {
                        check_max_length: false,
                        opts: opt.opts.clone(),
                        input: None,
                        output: Some(new_path), // where to restore
                        image: path.clone(),    // image to decode
                    },
                    mask,
                ) {
                    Ok(_) => std::fs::remove_file(path)?,
                    Err(err) => {
                        error!("error decoding {}: {:?}", path.display(), err);
                        continue;
                    }
                }
            }
        }
    } else {
        let image_client = image_api::PicsumClient::new();
        for (_, dirent) in std::fs::read_dir(&opt.dir)
            .context(format!("reading {:?}", opt.dir))?
            .into_iter()
            .filter(|r| r.is_ok())
            // SAFETY: since we only have the Ok variants from above `filter`
            .map(|r| r.unwrap())
            .filter(is_not_hidden)
            .enumerate()
        {
            if dirent.path().is_file() {
                let path = dirent.path();

                let mut new_fname: PathBuf = dirent.path().clone();
                new_fname.set_file_name(base64::encode(
                    new_fname.file_name().unwrap().to_str().unwrap(),
                ));
                new_fname.set_extension("png");

                debug!("encoding {} ==> {}", path.display(), new_fname.display());

                let bytes_to_mask = path.metadata()?.len();
                let width_of_img_to_request =
                    image_api::get_square_image_width_from_bytes(bytes_to_mask as usize);

                let mask = match image_client.get_square_image(width_of_img_to_request) {
                    Ok(data) => ImageReader::new(Cursor::new(data))
                        .with_guessed_format()?
                        .decode()?
                        .into_rgb8(),
                    Err(err) => {
                        error!(
                            "fetching image from api width={} | {:?}",
                            width_of_img_to_request, err
                        );
                        continue;
                    }
                };

                match encode(
                    cli::Encode {
                        check_max_length: false,
                        opts: opt.opts.clone(),
                        input: Some(dirent.path()), // what to hide
                        output: Some(new_fname),    // where to hide
                        image: PathBuf::new(),      // not used as calling `encode` directly
                    },
                    mask, // image to hide in
                ) {
                    Ok(_) => std::fs::remove_file(dirent.path())?,
                    Err(err) => {
                        error!("error encoding {}: {:?}", path.display(), err);
                        continue;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Determine whether a directory entry is a hidden file (i.e. starts with a `.`)
fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| !s.starts_with('.'))
        .unwrap_or(false)
}
