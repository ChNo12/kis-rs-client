use std::fmt;

#[cfg(feature = "websocket-client")]
use aes::Aes256;
#[cfg(feature = "websocket-client")]
use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
#[cfg(feature = "websocket-client")]
use base64::Engine;
#[cfg(feature = "websocket-client")]
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use crate::config::SecretString;
use crate::error::{Error, Result};
use crate::websocket::SystemMessage;

#[cfg(feature = "websocket-client")]
type Aes256CbcDecryptor = cbc::Decryptor<Aes256>;

const AES256_KEY_LEN: usize = 32;
const AES_CBC_IV_LEN: usize = 16;

#[derive(Clone, Eq, PartialEq)]
pub struct ExecutionNoticeCipher {
    key: SecretString,
    iv: SecretString,
}

impl ExecutionNoticeCipher {
    pub fn new(key: impl Into<String>, iv: impl Into<String>) -> Result<Self> {
        let key = SecretString::new(key);
        let iv = SecretString::new(iv);

        validate_cipher_material(key.expose_secret(), iv.expose_secret())?;

        Ok(Self { key, iv })
    }

    pub fn from_system_message(message: &SystemMessage) -> Result<Option<Self>> {
        let Some(key) = message.encryption_key() else {
            return Ok(None);
        };
        let Some(iv) = message.encryption_iv() else {
            return Ok(None);
        };

        Self::new(key, iv).map(Some)
    }

    #[cfg(feature = "websocket-client")]
    pub fn decrypt_base64(&self, encrypted_payload: &str) -> Result<String> {
        let encrypted = BASE64_STANDARD.decode(encrypted_payload).map_err(|error| {
            Error::parse(format!(
                "failed to decode execution notice payload as base64: {error}"
            ))
        })?;

        let decrypted = Aes256CbcDecryptor::new(
            self.key.expose_secret().as_bytes().into(),
            self.iv.expose_secret().as_bytes().into(),
        )
        .decrypt_padded_vec_mut::<Pkcs7>(&encrypted)
        .map_err(|error| {
            Error::parse(format!(
                "failed to decrypt execution notice payload: {error}"
            ))
        })?;

        String::from_utf8(decrypted).map_err(|error| {
            Error::parse(format!(
                "decrypted execution notice payload is not UTF-8: {error}"
            ))
        })
    }
}

impl fmt::Debug for ExecutionNoticeCipher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionNoticeCipher")
            .field("key", &"***")
            .field("iv", &"***")
            .finish()
    }
}

fn validate_cipher_material(key: &str, iv: &str) -> Result<()> {
    if key.len() != AES256_KEY_LEN {
        return Err(Error::config(
            "execution notice AES-256 key must be 32 bytes",
        ));
    }

    if iv.len() != AES_CBC_IV_LEN {
        return Err(Error::config(
            "execution notice AES-CBC IV must be 16 bytes",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_notice_cipher_rejects_invalid_key_or_iv_length() {
        assert_eq!(
            ExecutionNoticeCipher::new("short", "abcdef9876543210"),
            Err(Error::config(
                "execution notice AES-256 key must be 32 bytes"
            ))
        );
        assert_eq!(
            ExecutionNoticeCipher::new("0123456789abcdef0123456789abcdef", "short"),
            Err(Error::config(
                "execution notice AES-CBC IV must be 16 bytes"
            ))
        );
    }

    #[test]
    fn execution_notice_cipher_debug_masks_key_and_iv() {
        let cipher =
            ExecutionNoticeCipher::new("0123456789abcdef0123456789abcdef", "abcdef9876543210")
                .unwrap();

        let debug = format!("{cipher:?}");

        assert!(!debug.contains("0123456789abcdef0123456789abcdef"));
        assert!(!debug.contains("abcdef9876543210"));
        assert!(debug.contains("***"));
    }

    #[cfg(feature = "websocket-client")]
    #[test]
    fn decrypt_rejects_invalid_base64_payload() {
        let cipher =
            ExecutionNoticeCipher::new("0123456789abcdef0123456789abcdef", "abcdef9876543210")
                .unwrap();

        assert!(matches!(
            cipher.decrypt_base64("not base64"),
            Err(Error::Parse { .. })
        ));
    }
}
