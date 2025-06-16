use aes::cipher::{
    BlockEncrypt, BlockEncryptMut, KeyInit, KeyIvInit, block_padding::Pkcs7,
    generic_array::GenericArray,
};
use aes::{Aes128Enc, Aes192Enc, Aes256Enc};
use anyhow::{Result, anyhow, bail};
use base64::prelude::*;
use clap::{Args, ValueEnum};
use serde::{Deserialize, de};

use Modes::*;

type Aes128CbcEnc = cbc::Encryptor<Aes128Enc>;
type Aes192CbcEnc = cbc::Encryptor<Aes192Enc>;
type Aes256CbcEnc = cbc::Encryptor<Aes256Enc>;

#[derive(Args, Deserialize, Debug)]
pub struct Encryption {
    /// Send encrypted push. Make sure not to use encryption, use --no-encryption, simple as -E
    #[arg(long, short = 'e', overrides_with = "no_encrypt")]
    #[serde(default)]
    encrypt: bool,

    /// Encrypt modes
    #[arg(
        long,
        short = 'm',
        default_value = "aes256cbc",
        hide_possible_values = true
    )]
    #[serde(default)]
    modes: Modes,

    /// For encryption
    #[arg(long, visible_alias = "aeskey", value_name = "KEY")]
    #[serde(default)]
    aes_key: Option<String>,

    /// For encryption
    #[arg(long, visible_alias = "aesiv", value_name = "IV")]
    #[serde(default)]
    aes_iv: Option<String>,
}

impl Encryption {
    pub fn encrypted(&self) -> bool {
        self.encrypt
    }

    pub fn merge(&mut self, other: Self, no_encrypt: bool) {
        if !self.encrypt && !no_encrypt {
            self.encrypt = other.encrypt;
        }
        if self.aes_key.is_none() && other.aes_key.is_some() {
            self.aes_key = other.aes_key;
            self.aes_iv = other.aes_iv;
            self.modes = other.modes;
        }
    }

    fn key(&self, len: usize) -> Result<&[u8]> {
        self.aes_key
            .as_deref()
            .filter(|key| key.len() == len)
            .ok_or(anyhow!("The key length mismatched"))
            .map(|key| key.as_bytes())
            .or(Err(anyhow!("Missing key")))
    }

    fn iv(&self) -> Result<[u8; 16]> {
        let mut iv = [0; 16];
        for (i, b) in self
            .aes_iv
            .as_deref()
            .filter(|key| key.len() == 16)
            .ok_or(anyhow!("The iv length mismatched"))
            .map(|key| key.as_bytes())
            .or(Err(anyhow!("Missing iv")))?
            .iter()
            .enumerate()
        {
            iv[i] = *b;
        }
        Ok(iv)
    }

    fn aes_128_key(&self) -> Result<[u8; 16]> {
        let mut key = [0; 16];

        for (i, b) in self.key(16)?.iter().enumerate() {
            key[i] = *b;
        }

        Ok(key)
    }

    fn aes_192_key(&self) -> Result<[u8; 24]> {
        let mut key = [0; 24];

        for (i, b) in self.key(24)?.iter().enumerate() {
            key[i] = *b;
        }

        Ok(key)
    }

    fn aes_256_key(&self) -> Result<[u8; 32]> {
        let mut key = [0; 32];

        for (i, b) in self.key(32)?.iter().enumerate() {
            key[i] = *b;
        }

        Ok(key)
    }
}

#[inline]
fn base64_encode(data: &[u8]) -> String {
    BASE64_STANDARD.encode(data)
}

impl Encryption {
    pub fn is_valid(&self) -> Result<()> {
        if let Some(key) = self.aes_key.as_deref() {
            if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
                bail!("The key is alphanumeric only")
            }
            if self
                .aes_iv
                .as_deref()
                .map(|iv| !iv.chars().all(|c| c.is_ascii_alphanumeric()))
                .unwrap_or(false)
            {
                bail!("The iv is alphanumeric only")
            }

            match (
                self.modes,
                key.len(),
                self.aes_iv
                    .as_deref()
                    .map(|iv| iv.len() == 16)
                    .unwrap_or(false),
            ) {
                (Aes128Cbc, 16, true) => Ok(()),
                (Aes192Cbc, 24, true) => Ok(()),
                (Aes256Cbc, 32, true) => Ok(()),
                (Aes128Ecb, 16, _) => Ok(()),
                (Aes192Ecb, 24, _) => Ok(()),
                (Aes256Ecb, 32, _) => Ok(()),
                _ => bail!("Invalid key/or for modes {:?}", self.modes),
            }
        } else {
            bail!("Missing key for encryption")
        }
    }

    pub fn encrypt(&self, plain: &str) -> Result<String> {
        self.is_valid()?;
        let plain = plain.as_bytes();

        match self.modes {
            Modes::Aes128Ecb => {
                let key = GenericArray::from(self.aes_128_key()?);
                let cipher = Aes128Enc::new(&key);
                let mut block = GenericArray::clone_from_slice(plain);
                cipher.encrypt_block(&mut block);
                Ok(base64_encode(&block))
            }
            Modes::Aes192Ecb => {
                let key = GenericArray::from(self.aes_192_key()?);
                let cipher = Aes192Enc::new(&key);
                let mut block = GenericArray::clone_from_slice(plain);
                cipher.encrypt_block(&mut block);
                Ok(base64_encode(&block))
            }
            Modes::Aes256Ecb => {
                let key = GenericArray::from(self.aes_256_key()?);
                let cipher = Aes256Enc::new(&key);
                let mut block = GenericArray::clone_from_slice(plain);
                cipher.encrypt_block(&mut block);
                Ok(base64_encode(&block))
            }
            Modes::Aes128Cbc => {
                let cipher = Aes128CbcEnc::new(&self.aes_128_key()?.into(), &self.iv()?.into());
                let mut buffer = vec![0u8; plain.len() + 16];
                let pt_len = plain.len();
                buffer[..pt_len].copy_from_slice(plain);
                let ct = cipher
                    .encrypt_padded_mut::<Pkcs7>(&mut buffer, pt_len)
                    .or(Err(anyhow!("Failed to encrypt")))?;
                Ok(base64_encode(ct))
            }
            Modes::Aes192Cbc => {
                let cipher = Aes192CbcEnc::new(&self.aes_192_key()?.into(), &self.iv()?.into());
                let mut buffer = vec![0u8; plain.len() + 16];
                let pt_len = plain.len();
                buffer[..pt_len].copy_from_slice(plain);
                let ct = cipher
                    .encrypt_padded_mut::<Pkcs7>(&mut buffer, pt_len)
                    .or(Err(anyhow!("Failed to encrypt")))?;
                Ok(base64_encode(ct))
            }
            Modes::Aes256Cbc => {
                let cipher = Aes256CbcEnc::new(&self.aes_256_key()?.into(), &self.iv()?.into());
                let mut buffer = vec![0u8; plain.len() + 16];
                let pt_len = plain.len();
                buffer[..pt_len].copy_from_slice(plain);
                let ct = cipher
                    .encrypt_padded_mut::<Pkcs7>(&mut buffer, pt_len)
                    .or(Err(anyhow!("Failed to encrypt")))?;
                Ok(base64_encode(ct))
            }
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Default, Debug)]
enum Modes {
    #[clap(alias = "aes128cbc", alias = "aes128_cbc", alias = "aes_128_cbc")]
    Aes128Cbc,

    #[clap(alias = "aes192cbc", alias = "aes192_cbc", alias = "aes_192_cbc")]
    Aes192Cbc,

    #[default]
    #[clap(alias = "aes256cbc", alias = "aes256_cbc", alias = "aes_256_cbc")]
    Aes256Cbc,

    #[clap(alias = "aes128ecb", alias = "aes128_ecb", alias = "aes_128_ecb")]
    Aes128Ecb,

    #[clap(alias = "aes192ecb", alias = "aes192_ecb", alias = "aes_192_ecb")]
    Aes192Ecb,

    #[clap(alias = "aes256ecb", alias = "aes256_ecb", alias = "aes_256_ecb")]
    Aes256Ecb,
}

impl<'de> de::Deserialize<'de> for Modes {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CipherVisitor;

        impl<'de> de::Visitor<'de> for CipherVisitor {
            type Value = Modes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing MyEnum")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().replace('-', "_").as_str() {
                    "aes128cbc" | "aes128_cbc" | "aes_128_cbc" => Ok(Aes128Cbc),
                    "aes192cbc" | "aes192_cbc" | "aes_192_cbc" => Ok(Aes192Cbc),
                    "aes256cbc" | "aes256_cbc" | "aes_256_cbc" => Ok(Aes256Cbc),
                    "aes128ecb" | "aes128_ecb" | "aes_128_ecb" => Ok(Aes128Ecb),
                    "aes192ecb" | "aes192_ecb" | "aes_192_ecb" => Ok(Aes192Ecb),
                    "aes256ecb" | "aes256_ecb" | "aes_256_ecb" => Ok(Aes256Ecb),
                    _ => Err(de::Error::unknown_variant(
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
