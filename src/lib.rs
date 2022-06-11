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
