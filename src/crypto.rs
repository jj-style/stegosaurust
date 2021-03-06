use crate::CryptoError;

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
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let salt = SaltString::new(&s).map_err(|_| CryptoError::Salt)?;
    let password_hash = hash_password(key, &salt).map_err(|_| CryptoError::PasswordHash)?;
    let password_hash = password_hash.hash.unwrap();
    let (key, iv) = password_hash.as_bytes().split_at(32);
    let cipher = Aes128CbcEnc::new_from_slices(key, iv).map_err(CryptoError::Cipher)?;
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    let message = ["Salted__".as_bytes(), salt.as_bytes(), &ciphertext].concat();
    Ok(message)
}

/// Helper function to provide simple mechanism to decrypt some bytes with a key using AES-256-CBC.
///
/// Ciphertext is interoperable with openssl encryption format.
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if !ciphertext.starts_with(b"Salted__") {
        return Err(CryptoError::Decryption(
            "message was not encrypted when encoded".to_string(),
        ));
    }
    if ciphertext.len() < 16 {
        return Err(CryptoError::Decryption(
            "ciphertext is too short".to_string(),
        ));
    }
    let (_, rest) = ciphertext.split_at(8); //ignore prefix 'Salted__'
    let (s, rest) = rest.split_at(8);
    let s = String::from_utf8(s.to_vec()).map_err(|e| CryptoError::Decryption(format!("{}", e)))?;
    let salt = SaltString::new(&s).map_err(|_| CryptoError::Salt)?;
    let password_hash = hash_password(key, &salt).map_err(|_| CryptoError::PasswordHash)?;
    let password_hash = password_hash.hash.unwrap();
    let (key, iv) = password_hash.as_bytes().split_at(32);
    let cipher = Aes128CbcDec::new_from_slices(key, iv).map_err(CryptoError::Cipher)?;
    let plaintext = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(rest)
        .map_err(|e| CryptoError::Decryption(format!("{}", e)))?;
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

    #[test]
    fn test_decryption_fails_when_not_encrypted() {
        let plaintext = b"secret message";
        let key = b"rust";
        let result = decrypt(plaintext, key);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(format!("{}", err).contains("not encrypted"));
        }
    }

    #[test]
    fn test_invalid_short_cryptotext_not_panic() {
        let ciphertext = b"Salted__short";
        let key = b"rust";
        let result = decrypt(ciphertext, key);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(format!("{}", err).contains("too short"));
        }
    }
}
