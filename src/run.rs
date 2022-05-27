use std::io::{Write,Read,stdin,stdout};
use std::fs::File;

use anyhow::{Context,Result,bail};
use atty::Stream;
use image::io::Reader as ImageReader;
use image::{ImageFormat};
use base64;

use pbkdf2::{
    password_hash::{
        rand_core::{OsRng,RngCore},
        PasswordHasher, SaltString,Salt
    },
    Pbkdf2,Params
};
use aes::cipher::{block_padding::Pkcs7,KeyIvInit,BlockEncryptMut,BlockDecryptMut};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;


use crate::cli;
use crate::steganography::{Lsb,Steganography,END};

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

        if let Some(key) = &opt.key {
            // TODO: short message not encrypted but attempt to decrypt = panicc!
            let (salt, rest) = result.split_at(22);
            let salt = Salt::new(std::str::from_utf8(salt).unwrap()).unwrap();
            let (iv, rest) = rest.split_at(16);
            let password_hash = Pbkdf2.hash_password_customized(key.as_bytes(),
                None, None, 
                Params {
                    rounds: 10_000,
                    output_length: 16,
                },
                salt).unwrap();
            let cipher = Aes128CbcDec::new_from_slices(&password_hash.hash.unwrap().as_bytes(), &iv).unwrap();
            result = cipher.decrypt_padded_vec_mut::<Pkcs7>(&rest).context("decryption failed")?;
        }
        
        if opt.base64 || opt.key.is_some() {
            result = base64::decode(result).context("failed to decode as base64")?;
        }

        if let Some(path) = opt.output {
            let mut f = File::create(&path).context(format!("failed to create file: {}", path.to_str().unwrap()))?;
            f.write_all(&result).context("failed to write message to file")?;
        } else {
            let result = match String::from_utf8(result.clone()){
                Ok(s) => s,
                Err(_) => {
                    eprintln!("failed to convert message to utf8 string... encoding with base64");
                    base64::encode(result)
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
         if opt.base64 || opt.key.is_some() {
            message = base64::encode(&message).as_bytes().to_vec();
        }

        // perform transformations if necessary
        if let Some(key) = &opt.key {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = Pbkdf2.hash_password_customized(key.as_bytes(),
                None, None, 
                Params {
                    rounds: 10_000,
                    output_length: 16,
                },
                &salt).unwrap();
            let mut iv = [0u8; 16];
            OsRng.fill_bytes(&mut iv);
            let cipher = Aes128CbcEnc::new_from_slices(&password_hash.hash.unwrap().as_bytes(), &iv).unwrap();
            let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&message);
            // println!("iv={:?} ({})",hex::encode(iv), iv.len());
            // println!("salt={:?} ({})",salt, salt.len());
            // println!("key={:?} ({})",hex::encode(password_hash.hash.unwrap()), password_hash.hash.unwrap().len());
            // println!("{:?}", base64::encode(&ciphertext));
            message = [salt.as_bytes(),&iv,&ciphertext].concat();
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