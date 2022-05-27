use std::io::{Write,Read,stdin,stdout};
use std::fs::File;

use anyhow::{Context,Result,bail};
use atty::Stream;
use image::io::Reader as ImageReader;
use image::{ImageFormat};
use base64;
use rand::{distributions::Alphanumeric, Rng};

use pbkdf2::{
    password_hash::{
        PasswordHasher, SaltString
    },
    Pbkdf2,Params
};
use aes::cipher::{block_padding::Pkcs7,KeyIvInit,BlockEncryptMut,BlockDecryptMut};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes256>;


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
        
        // perform transformations if necessary, decode then decrypt
        if opt.base64 {
            result = base64::decode(result).context("failed to decode as base64")?;
        }

        if let Some(key) = opt.key {
            // TODO: short message not encrypted but attempt to decrypt = panicc!
            let (_, rest) = result.split_at(8);
            let (s, rest) = rest.split_at(8);
            let s = String::from_utf8(s.to_vec()).unwrap(); 
            let salt = SaltString::new(&s).unwrap();
            let password_hash = Pbkdf2.hash_password_customized(key.as_bytes(),
                None, None, 
                Params {
                    rounds: 10_000,
                    output_length: 48,
                },
                &salt).unwrap();
            let password_hash = password_hash.hash.unwrap();
            let (key,iv) = password_hash.as_bytes().split_at(32);
            let cipher = Aes128CbcDec::new_from_slices(&key, &iv).unwrap();
            result = cipher.decrypt_padded_vec_mut::<Pkcs7>(&rest).context("decryption failed")?;
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
                buf.trim().as_bytes().to_vec()
            }
        };
         
        // perform transformations if necessary, encrypt then encode
        if let Some(key) = opt.key {
            let s: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();
            let salt = SaltString::new(&s).unwrap();
            let password_hash = Pbkdf2.hash_password_customized(key.as_bytes(),
                None, None, 
                Params {
                    rounds: 10_000,
                    output_length: 48,
                },
                &salt
            ).unwrap();
            let password_hash = password_hash.hash.unwrap();
            let (key, iv) = password_hash.as_bytes().split_at(32);
            let cipher = Aes128CbcEnc::new_from_slices(key, iv).unwrap();
            let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(&message);
            message = ["Salted__".as_bytes(), &salt.as_bytes(), &ciphertext].concat();
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