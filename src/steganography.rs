use std::convert::From;
use image::{RgbImage,Pixel};
use anyhow::{Result,bail};
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

pub const END: &[u8] = b"$T3G";

/// Behaviour to encode a message into an image and decode the message back out
pub trait Steganography {
    fn encode(&mut self, img: &RgbImage, msg: &[u8]) -> Result<RgbImage>;
    fn decode(&mut self, img: &RgbImage) -> Result<Vec<u8>>;
}

#[derive(Clone)]
enum BitMask {
    One   = 0b0000_0001,
    Two   = 0b0000_0010,
    Four  = 0b0000_0100,
    Eight = 0b0000_1000,
}

impl From<u8> for BitMask {
    fn from(num: u8) -> Self {
        match num {
            1 => BitMask::One,
            2 => BitMask::Two,
            3 => BitMask::Four,
            4 => BitMask::Eight,
            other => panic!("cannot create bitmask from value {}", other)
        }
    }
}

pub trait BitEncoding {
    fn encode(&mut self, bit: &u8, color_val: &mut u8);
    fn decode(&mut self, color_val: &u8) -> u8;
}

pub struct BitEncoder {
    encoder: Box<dyn BitEncoding>
}

impl BitEncoder {
    pub fn new(encoder: Box<dyn BitEncoding>) -> Self {
        BitEncoder{
            encoder
        }
    }
}

/// Least Significant Bit Steganography Method
pub struct Lsb;

impl Lsb {
    pub fn new() -> Self {
        Lsb{}
    }
}

/// Random Significant Bit Steganography Method
pub struct Rsb {
    max: u8,
    rng: Pcg64
}

impl Rsb {
    pub fn new(max: u8, seed: &str) -> Self {
        let rng: Pcg64 = Seeder::from(seed).make_rng();
        Rsb{max, rng}
    }

    fn next_mask(&mut self) -> BitMask {
        let n: u8 = self.rng.gen_range(1..=self.max);
        BitMask::from(n)
    }
}

impl BitEncoding for Rsb {
    fn encode(&mut self, bit: &u8, color_val: &mut u8) {
        let mask  = self.next_mask();
        print!("{:08b}", *color_val);
        if *bit == 0 {
            print!(" &");
            *color_val &= !(mask.clone() as u8);
            println!(" {:08b} = {:08b}", !(mask as u8), *color_val);
        } else if *bit == 1 {
            print!(" |");
            *color_val |= mask.clone() as u8;
            println!(" {:08b} = {:08b}", mask as u8, *color_val);
        }
    }

    fn decode(&mut self, color_val: &u8) -> u8 {
        let mask  = self.next_mask();
        let c = color_val & mask as u8;
        if c > 0 { 1 } else { 0 }
    }
}

impl BitEncoding for Lsb {
    fn encode(&mut self, bit: &u8, color_val: &mut u8) {
        if *bit == 0 {
            *color_val &= !(BitMask::One as u8);
        } else if *bit == 1 {
            *color_val |= BitMask::One as u8;
        }
    }

    fn decode(&mut self, color_val: &u8) -> u8 {
        color_val & BitMask::One as u8
    }
}

impl Steganography for BitEncoder {
    fn encode(&mut self, img: &RgbImage, msg: &[u8]) -> Result<RgbImage> {
        let msg = [msg, END].concat();

        let mut binary_msg = String::with_capacity(msg.len()*8);
        for byte in msg {
            binary_msg += &format!("{:08b}", byte);
        }
        let binary_msg: Vec<u8> = binary_msg.chars().map(|c| c.to_digit(10).unwrap() as u8).collect();
        
        let mut img = img.clone();
        
        let mut ctr = 0;
        for chunk in binary_msg.chunks(3) {
            let x = ctr % img.width();
            let y = ctr / img.width();
            let pixel = img.get_pixel_mut(x, y);
            for (idx, bit) in chunk.into_iter().enumerate() {
                self.encoder.encode(bit, &mut pixel[idx]);
            } 
            ctr+=1;
        }
        Ok(img)
    }

    fn decode(&mut self, img: &RgbImage) -> Result<Vec<u8>> {
        let mut bitstream: Vec<u8> = Vec::new();
        
        let mut endstream = String::new();
        for byte in END {
            endstream += &format!("{:08b}", byte);
        }

        let end = endstream.chars().map(|c| c.to_digit(10).unwrap() as u8).collect::<Vec<u8>>();
        
        'outer: for (_,_,pixel) in img.enumerate_pixels() {
            for value in pixel.channels() {
                if bitstream.iter().rev().take(end.len()).rev().map(|v| *v).collect::<Vec<u8>>().iter().eq(end.iter()) {
                    break 'outer;
                }
                bitstream.push(self.encoder.decode(value));
            }
        }

        if bitstream.iter().rev().take(end.len()).rev().map(|v| *v).collect::<Vec<u8>>().iter().ne(end.iter()) {
            bail!("encoded message could not be found in the image");
        }

        // message found in the bitstream, remove the END indicator
        bitstream.truncate(bitstream.len() - end.len());
        let mut msg = Vec::new();
        for chrs in bitstream.chunks(8) {
            let binval = u8::from_str_radix(
                &chrs.iter()
                .map(|c| format!{"{}",c})
                .collect::<Vec<String>>()
                .join(""), 2)
                .expect("not a binary number");
            msg.push(binval);
        }
        Ok(msg)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsb_steganography() {
        let img = RgbImage::new(32, 32);
        let lsb = Box::new(Lsb::new());
        let mut enc: Box<dyn Steganography> = Box::from(BitEncoder::new(lsb));
        let secret_message = "🦕 hiding text!".as_bytes();
        let encoded: RgbImage = enc.encode(&img, secret_message).unwrap();
        assert_eq!(enc.decode(&encoded).unwrap(), secret_message);
    }

    #[test]
    fn test_rsb_steganography() {
        let img = RgbImage::new(32, 32);
        let rsb_enc = Box::new(Rsb::new(2, "seed"));
        let mut enc: Box<dyn Steganography> = Box::from(BitEncoder::new(rsb_enc));
        let rsb_dec = Box::new(Rsb::new(2, "seed"));
        let mut dec: Box<dyn Steganography> = Box::from(BitEncoder::new(rsb_dec));
        let secret_message = "🦕 hiding text!".as_bytes();
        let encoded: RgbImage = enc.encode(&img, secret_message).unwrap();
        assert_eq!(dec.decode(&encoded).unwrap(), secret_message);
    }
    
    #[test]
    fn test_rsb_random_determined_from_seed() {
        let mut rsb1 = Rsb::new(2, "seed");
        let mut rsb2 = Rsb::new(2, "seed");
        for _ in 0..10 {
            assert_eq!(rsb1.rng.gen::<u8>(), rsb2.rng.gen::<u8>());
        } 
    }

    #[test]
    fn test_rsb_random_determined_from_seed_different() {
        let mut rsb1 = Rsb::new(2, "seed");
        let mut rsb2 = Rsb::new(2, "seeb");
        let it = 1000;
        let mut matches = Vec::with_capacity(it);
        for _ in 0..it {
            matches.push(rsb1.rng.gen::<u8>() == rsb2.rng.gen::<u8>());
        }
        assert!(matches.contains(&false));
    }
    
    #[test]
    fn test_rsb_1_decrypts_with_lsb() {
        let img = RgbImage::new(32, 32);
        let rsb = Box::new(Rsb::new(1, "seed"));
        let mut rsb_enc: Box<dyn Steganography> = Box::from(BitEncoder::new(rsb));
        let lsb = Box::new(Lsb::new());
        let mut lsb_enc: Box<dyn Steganography> = Box::from(BitEncoder::new(lsb));

        let secret_message = "🦕 hiding text!".as_bytes();
        let encoded: RgbImage = rsb_enc.encode(&img, secret_message).unwrap();
        assert_eq!(lsb_enc.decode(&encoded).unwrap(), secret_message);
    }

    #[test]
    #[should_panic(expected="encoded message could not be found in the image")]
    fn test_rsb_3_not_decrypts_with_lsb() {
        let img = RgbImage::new(32, 32);
        let rsb = Box::new(Rsb::new(3, "seed"));
        let mut rsb_enc: Box<dyn Steganography> = Box::from(BitEncoder::new(rsb));
        let lsb = Box::new(Lsb::new());
        let mut lsb_enc: Box<dyn Steganography> = Box::from(BitEncoder::new(lsb));

        let secret_message = "🦕 hiding text!".as_bytes();
        let encoded: RgbImage = rsb_enc.encode(&img, secret_message).unwrap();
        lsb_enc.decode(&encoded).unwrap();
    }
    
}