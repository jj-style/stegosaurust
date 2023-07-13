//! Stegosaurust is a command line program and library to perform steganography in rust.

/// Data structures for parsing and validating command line options to `stegosaurust`.
pub mod cli;

mod run;
pub use run::run;

/// Compression module with a simple interface to compression/decomporession functions, without all the fuss.
pub mod compress;
/// Cryptography module to provide a simple interface to
/// common encryption and hashing function, without all the fuss.
pub mod crypto;
/// Steganography module containing different implementations of encoding methods.
pub mod steganography;

/// Image API contains an HTTP client to fetch images from a remote API
pub mod image_api;

use thiserror::Error;
#[derive(Error, Debug, Eq, PartialEq)]
pub enum StegError {
    #[error("Encoded message not found in data")]
    EncodingNotFound,
    #[error("Error decoding message: `{0}`")]
    Decoding(String),
    #[error("Compression error")]
    Compression(#[from] CompressionError),
    #[error("Encryption error")]
    Crypto(#[from] CryptoError),
    #[error("Unknown steganography error")]
    Unknown,
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum CompressionError {
    #[error("Compression error")]
    Compression(#[from] compression::prelude::CompressionError),
    #[error("Decompression error")]
    Decompression(#[from] compression::prelude::BZip2Error),
    #[error("empty data")]
    EmptyData,
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum CryptoError {
    #[error("Failed to get random salt")]
    Salt,
    #[error("Failed to hash password")]
    PasswordHash,
    #[error("Error creating cipher")]
    Cipher(#[from] aes::cipher::InvalidLength),
    #[error("Error decrypting ciphertext: `{0}`")]
    Decryption(String),
    #[error("unknown cryptography error")]
    Unknown,
}
