use image::RgbImage;
use anyhow::Result;

/// Behaviour to encode a message into an image and decode the message back out
pub trait Steganography {
    fn encode(&self, img: &RgbImage, msg: &str) -> Result<RgbImage>;
    fn decode(&self, img: &RgbImage) -> Result<String>;
}

/// Least Significant Bit Steganography Method
pub struct Lsb;

impl Lsb {
    pub fn new() -> Self {
        Lsb{}
    }
}

impl Steganography for Lsb {
    fn encode(&self, img: &RgbImage, msg: &str) -> Result<RgbImage> {
        let mut binary_msg = String::with_capacity(msg.len()*7);
        // TODO: map this to enum, or better just parse to int (0,1)
        for byte in msg.as_bytes() {
            binary_msg += &format!("{:b}", byte);
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
                    pixel[idx] &= bit;
                } else if *bit == 1 {
                    pixel[idx] |= bit;
                }
            } 
            ctr+=1;
        }
        Ok(img)
    }

    fn decode(&self, img: &RgbImage) -> Result<String> {
        todo!("implement decoding")
    }
}