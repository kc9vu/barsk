use base64::prelude::{Engine, BASE64_STANDARD};
use clap::ValueEnum;
use crypto::{
    aes::{self, KeySize},
    blockmodes::PkcsPadding,
    buffer::{ReadBuffer, WriteBuffer},
};
use serde::de::{Deserialize, Error, Visitor};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Cipher {
    Aes128Cbc,
    Aes192Cbc,
    Aes256Cbc,
    Aes128Ecb,
    Aes192Ecb,
    Aes256Ecb,
}

fn check_key(key: &[u8], cipher: Cipher) -> Result<(), String> {
    match cipher {
        Cipher::Aes128Cbc | Cipher::Aes128Ecb if key.len() == 16 => Ok(()),
        Cipher::Aes192Cbc | Cipher::Aes192Ecb if key.len() == 24 => Ok(()),
        Cipher::Aes256Cbc | Cipher::Aes256Ecb if key.len() == 32 => Ok(()),
        _ => Err("The key does not meet the requirements".into()),
    }
}

fn check_iv(iv: &str) -> Result<(), String> {
    if iv.len() == 16 {
        Ok(())
    } else {
        Err("The iv does not meet the requirements".into())
    }
}

pub fn encrypt(plain: &str, key: &str, iv: Option<&str>, cipher: Cipher) -> Result<String, String> {
    let key = key.as_bytes();
    check_key(key, cipher)?;
    if let Some(v) = iv {
        check_iv(v)?;
    }

    let plain = plain.as_bytes();
    assert!(plain.len() < 4096);

    let mut encryptor = match cipher {
        Cipher::Aes128Cbc => aes::cbc_encryptor(
            KeySize::KeySize128,
            key,
            iv.unwrap().as_bytes(),
            PkcsPadding,
        ),
        Cipher::Aes192Cbc => aes::cbc_encryptor(
            KeySize::KeySize192,
            key,
            iv.unwrap().as_bytes(),
            PkcsPadding,
        ),
        Cipher::Aes256Cbc => aes::cbc_encryptor(
            KeySize::KeySize256,
            key,
            iv.unwrap().as_bytes(),
            PkcsPadding,
        ),
        Cipher::Aes128Ecb => aes::ecb_encryptor(KeySize::KeySize128, key, PkcsPadding),
        Cipher::Aes192Ecb => aes::ecb_encryptor(KeySize::KeySize192, key, PkcsPadding),
        Cipher::Aes256Ecb => aes::ecb_encryptor(KeySize::KeySize256, key, PkcsPadding),
    };
    let mut buffer = [0; 4096];
    let mut read_buffer = crypto::buffer::RefReadBuffer::new(plain);
    let mut write_buffer = crypto::buffer::RefWriteBuffer::new(&mut buffer);

    if encryptor
        .encrypt(&mut read_buffer, &mut write_buffer, true)
        .is_err()
    {
        return Err("Failed encrypt message".into());
    }
    let mut binding = write_buffer.take_read_buffer();
    let cipher = binding.take_remaining();
    Ok(BASE64_STANDARD.encode(cipher))
}

impl<'de> Deserialize<'de> for Cipher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CipherVisitor;

        impl<'de> Visitor<'de> for CipherVisitor {
            type Value = Cipher;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing MyEnum")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value.to_lowercase().replace('-', "_").as_str() {
                    "aes128cbc" | "aes128_cbc" | "aes_128_cbc" => Ok(Cipher::Aes128Cbc),
                    "aes192cbc" | "aes192_cbc" | "aes_192_cbc" => Ok(Cipher::Aes192Cbc),
                    "aes256cbc" | "aes256_cbc" | "aes_256_cbc" => Ok(Cipher::Aes256Cbc),
                    "aes128ecb" | "aes128_ecb" | "aes_128_ecb" => Ok(Cipher::Aes128Ecb),
                    "aes192ecb" | "aes192_ecb" | "aes_192_ecb" => Ok(Cipher::Aes192Ecb),
                    "aes256ecb" | "aes256_ecb" | "aes_256_ecb" => Ok(Cipher::Aes256Ecb),
                    _ => Err(Error::unknown_variant(
                        value,
                        &[
                            "aes128cbc",
                            "aes192cbc",
                            "aes256cbc",
                            "aes128ecb",
                            "aes192ecb",
                            "aes256ecb",
                        ],
                    )),
                }
            }
        }
        deserializer.deserialize_str(CipherVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let plain = r#"{"body": "test", "sound": "birdsong"}"#;
        let key = "1234567890123456";
        let iv = "1111111111111111";
        let cipher = Cipher::Aes128Cbc;

        assert_eq!(
            encrypt(plain, key, Some(iv), cipher).unwrap(),
            "d3QhjQjP5majvNt5CjsvFWwqqj2gKl96RFj5OO+u6ynTt7lkyigDYNA3abnnCLpr".to_string()
        );
    }
}
