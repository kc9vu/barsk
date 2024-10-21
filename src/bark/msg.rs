use base64::prelude::{Engine, BASE64_STANDARD};
use clap::ValueEnum;
use crypto::{
    aes::{self, KeySize},
    blockmodes::PkcsPadding,
    buffer::{ReadBuffer, WriteBuffer},
};
use serde::{
    de::{Deserialize, Error, Visitor},
    Serialize,
};

use crate::command::Level;
use Method::*;

#[derive(Serialize, Debug)]
pub struct Msg<'a> {
    pub body: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_copy: Option<()>,

    #[serde(
        rename = "isArchive",
        skip_serializing_if = "Option::is_none",
        serialize_with = "super::de::serialize_archive"
    )]
    pub archive: Option<bool>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "super::de::serialize_level"
    )]
    pub level: Option<Level>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<&'a str>,

    #[serde(
        skip_serializing_if = "super::de::is_false",
        serialize_with = "super::de::serialize_call"
    )]
    pub call: bool,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // icon: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Method {
    #[clap(alias = "aes128cbc", alias = "aes128_cbc", alias = "aes_128_cbc")]
    Aes128Cbc,

    #[clap(alias = "aes192cbc", alias = "aes192_cbc", alias = "aes_192_cbc")]
    Aes192Cbc,

    #[clap(alias = "aes256cbc", alias = "aes256_cbc", alias = "aes_256_cbc")]
    Aes256Cbc,

    #[clap(alias = "aes128ecb", alias = "aes128_ecb", alias = "aes_128_ecb")]
    Aes128Ecb,

    #[clap(alias = "aes192ecb", alias = "aes192_ecb", alias = "aes_192_ecb")]
    Aes192Ecb,

    #[clap(alias = "aes256ecb", alias = "aes256_ecb", alias = "aes_256_ecb")]
    Aes256Ecb,
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CipherVisitor;

        impl<'de> Visitor<'de> for CipherVisitor {
            type Value = Method;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing MyEnum")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value.to_lowercase().replace('-', "_").as_str() {
                    "aes128cbc" | "aes128_cbc" | "aes_128_cbc" => Ok(Aes128Cbc),
                    "aes192cbc" | "aes192_cbc" | "aes_192_cbc" => Ok(Aes192Cbc),
                    "aes256cbc" | "aes256_cbc" | "aes_256_cbc" => Ok(Aes256Cbc),
                    "aes128ecb" | "aes128_ecb" | "aes_128_ecb" => Ok(Aes128Ecb),
                    "aes192ecb" | "aes192_ecb" | "aes_192_ecb" => Ok(Aes192Ecb),
                    "aes256ecb" | "aes256_ecb" | "aes_256_ecb" => Ok(Aes256Ecb),
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

pub fn is_valid_cipher(method: Method, key: &str, iv: Option<&str>) -> Result<(), &'static str> {
    match (
        method,
        key.len(),
        match iv {
            Some(iv) => iv.len() == 16,
            None => false,
        },
    ) {
        (Aes128Cbc, 16, true) => Ok(()),
        (Aes192Cbc, 24, true) => Ok(()),
        (Aes256Cbc, 32, true) => Ok(()),
        (Aes128Ecb, 16, _) => Ok(()),
        (Aes192Ecb, 24, _) => Ok(()),
        (Aes256Ecb, 32, _) => Ok(()),
        _ => Err("Check aes_key and/or aes_iv"),
    }
}

/// Encrypt plain text using AES encryption with CBC or ECB mode.
/// Only accepts plain length less than 4096.
pub fn encrypt(
    plain: &str,
    key: &str,
    iv: Option<&str>,
    method: Method,
) -> Result<String, crypto::symmetriccipher::SymmetricCipherError> {
    let key = key.as_bytes();
    let plain = plain.as_bytes();
    assert!(plain.len() < 4096);

    let mut encryptor = match method {
        Aes128Cbc => aes::cbc_encryptor(
            KeySize::KeySize128,
            key,
            iv.unwrap().as_bytes(),
            PkcsPadding,
        ),
        Aes192Cbc => aes::cbc_encryptor(
            KeySize::KeySize192,
            key,
            iv.unwrap().as_bytes(),
            PkcsPadding,
        ),
        Aes256Cbc => aes::cbc_encryptor(
            KeySize::KeySize256,
            key,
            iv.unwrap().as_bytes(),
            PkcsPadding,
        ),
        Aes128Ecb => aes::ecb_encryptor(KeySize::KeySize128, key, PkcsPadding),
        Aes192Ecb => aes::ecb_encryptor(KeySize::KeySize192, key, PkcsPadding),
        Aes256Ecb => aes::ecb_encryptor(KeySize::KeySize256, key, PkcsPadding),
    };
    let mut buffer = [0; 4096];
    let mut read_buffer = crypto::buffer::RefReadBuffer::new(plain);
    let mut write_buffer = crypto::buffer::RefWriteBuffer::new(&mut buffer);

    encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;
    let mut binding = write_buffer.take_read_buffer();
    let cipher = binding.take_remaining();
    Ok(BASE64_STANDARD.encode(cipher))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let plain = r#"{"body": "test", "sound": "birdsong"}"#;
        let key = "1234567890123456";
        let iv = "1111111111111111";
        let method = Aes128Cbc;

        assert_eq!(
            encrypt(plain, key, Some(iv), method).unwrap(),
            "d3QhjQjP5majvNt5CjsvFWwqqj2gKl96RFj5OO+u6ynTt7lkyigDYNA3abnnCLpr".to_string()
        );
    }
}
