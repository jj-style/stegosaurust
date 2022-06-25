use image::{Pixel, RgbImage};
use itertools_num::linspace;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::convert::From;

use crate::cli::BitDistribution;
use crate::StegError;

const END: &[u8] = b"$T3G";

/// Behaviour to encode a message into an image and decode the message back out
pub trait Steganography {
    /// Encodes a message into an image
    fn encode(&mut self, img: &RgbImage, msg: &[u8]) -> Result<RgbImage, StegError>;
    /// Decodes a message from an image
    fn decode(&mut self, img: &RgbImage) -> Result<Vec<u8>, StegError>;
    /// Computes the maximum length message that can be encoded into a given image with the steganography method implemented
    fn max_len(&self, img: &RgbImage) -> usize;
}

/// Bit masks for setting/clearing bits in bytes.
#[derive(Clone)]
enum BitMask {
    One = 0b0000_0001,
    Two = 0b0000_0010,
    Four = 0b0000_0100,
    Eight = 0b0000_1000,
}

impl From<u8> for BitMask {
    fn from(num: u8) -> Self {
        match num {
            1 => BitMask::One,
            2 => BitMask::Two,
            3 => BitMask::Four,
            4 => BitMask::Eight,
            other => panic!("cannot create bitmask from value {}", other),
        }
    }
}

/// Behvaiour to encode a single bit of information into a byte
pub trait BitEncoding {
    /// Encode a bit of information into a byte
    fn encode(&mut self, bit: &u8, color_val: &mut u8);
    /// Decode a bit of information from a byte
    fn decode(&mut self, color_val: &u8) -> u8;
}

/// A `BitEncoder` is something that can perform `BitEncoding`
pub struct BitEncoder {
    encoder: Box<dyn BitEncoding>,
    /// Bit distribution method to use when encoding bits
    pub bit_dist: BitDistribution,
}

impl BitEncoder {
    pub fn new(encoder: Box<dyn BitEncoding>, bd: Option<BitDistribution>) -> Self {
        BitEncoder {
            encoder,
            bit_dist: bd.unwrap_or_default(),
        }
    }
}

/// Least significant bit encoding
///
/// With a binary message, each bit of the message is encoded
/// into the least significant bit of each RGB byte of each pixel.
pub struct Lsb;

impl Lsb {
    /// Creates an new instance of `Lsb`
    pub fn new() -> Self {
        Lsb {}
    }
}

impl Default for Lsb {
    fn default() -> Self {
        Lsb::new()
    }
}

/// Random significant bit encoding
///
/// With a binary message, each bit of the message is encoded
/// randomly into one of the `n` least significant bits of each RGB byte of each pixel.
pub struct Rsb {
    /// The maximum significant bit to possibly set/clear when encoding (1-4)
    max: u8,
    /// A seeded random number generator do determine which significant bit to encode to/decode from
    rng: Pcg64,
}

impl Rsb {
    /// Creates an new instance of `Rsb`
    pub fn new(max: u8, seed: &str) -> Self {
        let rng: Pcg64 = Seeder::from(seed).make_rng();
        Rsb { max, rng }
    }

    /// Randomly choose the next `BitMask` for encoding/decoding the next bit
    fn next_mask(&mut self) -> BitMask {
        let n: u8 = self.rng.gen_range(1..=self.max);
        BitMask::from(n)
    }
}

impl BitEncoding for Rsb {
    fn encode(&mut self, bit: &u8, color_val: &mut u8) {
        let mask = self.next_mask();
        if *bit == 0 {
            *color_val &= !(mask as u8);
        } else if *bit == 1 {
            *color_val |= mask as u8;
        }
    }

    fn decode(&mut self, color_val: &u8) -> u8 {
        let mask = self.next_mask();
        let c = color_val & mask as u8;
        if c > 0 {
            1
        } else {
            0
        }
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
    fn max_len(&self, img: &RgbImage) -> usize {
        ((img.width() * img.height() * 3) as usize - (END.len() * 8)) / 8
    }

    fn encode(&mut self, img: &RgbImage, msg: &[u8]) -> Result<RgbImage, StegError> {
        let msg = [msg, END].concat();

        let mut binary_msg = String::with_capacity(msg.len() * 8);
        for byte in msg {
            binary_msg += &format!("{:08b}", byte);
        }
        let binary_msg: Vec<u8> = binary_msg
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect();

        let mut img = img.clone();

        for (ctr, chunk) in binary_msg.chunks(3).enumerate() {
            match self.bit_dist {
                BitDistribution::Sequential => {
                    let x = ctr as u32 % img.width();
                    let y = ctr as u32 / img.width();
                    let pixel = img.get_pixel_mut(x, y);
                    for (idx, bit) in chunk.iter().enumerate() {
                        self.encoder.encode(bit, &mut pixel[idx]);
                    }
                }
                BitDistribution::Linear => todo!("implement linear distribution encoding"),
            }
        }
        Ok(img)
    }

    fn decode(&mut self, img: &RgbImage) -> Result<Vec<u8>, StegError> {
        let mut bitstream: Vec<u8> = Vec::new();

        let mut endstream = String::new();
        for byte in END {
            endstream += &format!("{:08b}", byte);
        }

        let end = endstream
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect::<Vec<u8>>();

        match self.bit_dist {
            BitDistribution::Sequential => {
                'outer: for (_, _, pixel) in img.enumerate_pixels() {
                    for value in pixel.channels() {
                        if has_end(&bitstream, &end) {
                            break 'outer;
                        }
                        bitstream.push(self.encoder.decode(value));
                    }
                }
            }
            BitDistribution::Linear => todo!("implement linear distribution decoding"),
        }

        if !has_end(&bitstream, &end) {
            return Err(StegError::EncodingNotFound);
        }

        // message found in the bitstream, remove the END indicator
        bitstream.truncate(bitstream.len() - end.len());
        let mut msg = Vec::new();
        for chrs in bitstream.chunks(8) {
            let binval = u8::from_str_radix(
                &chrs
                    .iter()
                    .map(|c| format! {"{}",c})
                    .collect::<Vec<String>>()
                    .join(""),
                2,
            )
            .map_err(|e| StegError::Decoding(format!("reconstructing byte: {}", e)))?;
            msg.push(binval);
        }
        Ok(msg)
    }
}

/// determines if a stream of `byte`s has a terminating `end` sequence of bytes
/// 
/// # Example
/// ```rust
/// use stegosaurust::steganography::has_end;
/// let bytes_1 = [1, 2, 3];
/// let bytes_2 = [1, 2, 2];
/// let end = [2, 3];
/// 
/// assert_eq!(has_end(&bytes_1, &end), true);
/// assert_eq!(has_end(&bytes_2, &end), false);
/// ```
pub fn has_end(bytes: &[u8], end: &[u8]) -> bool {
    bytes
        .iter()
        .rev()
        .take(end.len())
        .rev()
        .copied()
        .collect::<Vec<u8>>()
        .iter()
        .eq(end.iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsb_steganography() {
        let img = RgbImage::new(32, 32);
        let lsb = Box::new(Lsb::default());
        let mut enc: Box<dyn Steganography> = Box::from(BitEncoder::new(lsb, None));
        let secret_message = "🦕 hiding text!".as_bytes();
        let encoded: RgbImage = enc.encode(&img, secret_message).unwrap();
        assert_eq!(enc.decode(&encoded).unwrap(), secret_message);
    }

    #[test]
    fn test_rsb_steganography() {
        let img = RgbImage::new(32, 32);
        let rsb_enc = Box::new(Rsb::new(2, "seed"));
        let mut enc: Box<dyn Steganography> = Box::from(BitEncoder::new(rsb_enc, None));
        let rsb_dec = Box::new(Rsb::new(2, "seed"));
        let mut dec: Box<dyn Steganography> = Box::from(BitEncoder::new(rsb_dec, None));
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
        let mut rsb_enc: Box<dyn Steganography> = Box::from(BitEncoder::new(rsb, None));
        let lsb = Box::new(Lsb::default());
        let mut lsb_enc: Box<dyn Steganography> = Box::from(BitEncoder::new(lsb, None));

        let secret_message = "🦕 hiding text!".as_bytes();
        let encoded: RgbImage = rsb_enc.encode(&img, secret_message).unwrap();
        assert_eq!(lsb_enc.decode(&encoded).unwrap(), secret_message);
    }

    #[test]
    fn test_rsb_3_not_decrypts_with_lsb() {
        let img = RgbImage::new(32, 32);
        let rsb = Box::new(Rsb::new(3, "seed"));
        let mut rsb_enc: Box<dyn Steganography> = Box::from(BitEncoder::new(rsb, None));
        let lsb = Box::new(Lsb::default());
        let mut lsb_enc: Box<dyn Steganography> = Box::from(BitEncoder::new(lsb, None));

        let secret_message = "🦕 hiding text!".as_bytes();
        let encoded: RgbImage = rsb_enc.encode(&img, secret_message).unwrap();

        let result = lsb_enc.decode(&encoded);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StegError::EncodingNotFound);
    }
}
