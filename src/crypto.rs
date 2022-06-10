use anyhow::Result;

use rand::{distributions::Alphanumeric, Rng};

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, SaltString},
    Params, Pbkdf2,
};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes256>;

/// Helper function to provide simple mechanism to encrypt some bytes with a key using AES-256-CBC.
///
/// Output is interoperable with openssl encryption format.
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let salt = SaltString::new(&s).unwrap();
    let password_hash = hash_password(key, &salt).unwrap();
    let password_hash = password_hash.hash.unwrap();
    let (key, iv) = password_hash.as_bytes().split_at(32);
    let cipher = Aes128CbcEnc::new_from_slices(key, iv).unwrap();
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    let message = ["Salted__".as_bytes(), salt.as_bytes(), &ciphertext].concat();
    Ok(message)
}

/// Helper function to provide simple mechanism to decrypt some bytes with a key using AES-256-CBC.
///
/// Ciphertext is interoperable with openssl encryption format.
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    // TODO: short message not encrypted but attempt to decrypt = panic!
    let (_, rest) = ciphertext.split_at(8); //ignore prefix 'Salted__'
    let (s, rest) = rest.split_at(8);
    let s = String::from_utf8(s.to_vec()).unwrap();
    let salt = SaltString::new(&s).unwrap();
    let password_hash = hash_password(key, &salt).unwrap();
    let password_hash = password_hash.hash.unwrap();
    let (key, iv) = password_hash.as_bytes().split_at(32);
    let cipher = Aes128CbcDec::new_from_slices(key, iv).unwrap();
    let plaintext = cipher.decrypt_padded_vec_mut::<Pkcs7>(rest)?;
    Ok(plaintext)
}

/// Convenience function to hash password and salt
/// to generate key for use with AES-256 encryption.
///
/// Uses PBKDF2 with 10,000 rounds of SHA256 hashing to generate a 48-byte response.
/// 48-byte response contains the 16-byte IV and 32-byte key.
pub fn hash_password<'a>(
    key: &'a [u8],
    salt: &'a SaltString,
) -> Result<PasswordHash<'a>, pbkdf2::password_hash::Error> {
    Pbkdf2.hash_password_customized(
        key,
        None,
        None,
        Params {
            rounds: 10_000,
            output_length: 48,
        },
        salt,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_256() {
        let plaintext = b"secret message";
        let key = b"rust";
        let ciphertext = encrypt(plaintext, key).unwrap();
        assert!(decrypt(&ciphertext, key)
            .unwrap()
            .iter()
            .eq(plaintext.iter()));
    }
}
