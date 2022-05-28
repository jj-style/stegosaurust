use image::{RgbImage,Pixel};
use anyhow::{Result,bail};

pub const END: &[u8] = b"$T3G";

/// Behaviour to encode a message into an image and decode the message back out
pub trait Steganography {
    fn encode(&self, img: &RgbImage, msg: &[u8]) -> Result<RgbImage>;
    fn decode(&self, img: &RgbImage) -> Result<Vec<u8>>;
}

/// Least Significant Bit Steganography Method
pub struct Lsb;

impl Lsb {
    pub fn new() -> Self {
        Lsb{}
    }
}

impl Steganography for Lsb {
    fn encode(&self, img: &RgbImage, msg: &[u8]) -> Result<RgbImage> {
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
                if *bit == 0 {
                    pixel[idx] &= 0b1111_1110;
                } else if *bit == 1 {
                    pixel[idx] |= 0b0000_0001;
                }
            } 
            ctr+=1;
        }
        Ok(img)
    }

    fn decode(&self, img: &RgbImage) -> Result<Vec<u8>> {
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
                bitstream.push(value & 0b0000_0001);
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
        // Ok(String::from_utf8(msg)?.as_bytes().to_vec())
        Ok(msg)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsb_steganography() {
        let img = RgbImage::new(32, 32);
        let lsb = Lsb{};
        let secret_message = "🦕 hiding text!".as_bytes();
        let encoded: RgbImage = lsb.encode(&img, secret_message).unwrap();
        assert_eq!(lsb.decode(&encoded).unwrap(), secret_message);
    }
    
}