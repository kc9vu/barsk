use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{ArgAction, ArgGroup, Parser, ValueEnum};
use serde::{de, Deserialize, Serialize};

use crate::bark::{
    self,
    msg::{Method, Msg},
};

#[derive(Debug, Parser)]
#[command(
    author = "kc9vu",
    about = "Push to your iPhone with text, links and more!",
    version,
    max_term_width = 80
)]
#[command(
    group(
    ArgGroup::new("push level")
        .args(["active", "time_sensitive", "passive"])
        .conflicts_with("level")
))]
pub struct Cli {
    /// Push content
    pub body: String,

    /// The title will be shown on the notification bar
    #[arg(long, short)]
    pub title: Option<String>,

    /// The content will be copied to clipboard or <BODY>
    #[arg(long, short)]
    pub copy: Option<String>,

    /// Auto copy the content to clipboard. Available under iOS 14.5
    #[arg(long, short = 'C')]
    pub auto_copy: bool,

    /// The URL will be opened when the notification is clicked
    #[arg(long)]
    pub url: Option<String>,

    /// Rings continuously for 30 seconds
    #[arg(alias = "persistent_ringing", long, short = 'r')]
    pub call: bool,

    /// The icon url will be shown in the notification bar. Available above iOS 15
    #[arg(long)]
    pub icon: Option<String>,

    /// The badge number will be shown in the app icon
    #[arg(long, short)]
    pub badge: Option<String>,

    #[command(flatten)]
    pub conf: Conf,

    /// The config file path
    // Use ArgAction::Set can use multiple config_file, the last one will be used
    #[arg(env = "BARSK_CONFIG_PATH", long = "config", short = 'F', action = ArgAction::Set)]
    pub config_file: Option<PathBuf>,

    /// Don't use any config file, it means run without adding any unspecified arguments
    #[arg(long, short = 'z')]
    pub thats_all: bool,

    /// Print message instead of sending it
    #[arg(long, short = 'p')]
    pub dry_run: bool,
}

#[derive(Debug, Default, Deserialize, Parser)]
pub struct Conf {
    /// The server url for push message
    #[arg(long, short)]
    pub server: Option<String>,

    /// The device key for push message
    #[arg(long, short)]
    pub device_key: Option<String>,

    /// Archiving the message to history
    #[serde(default)]
    #[arg(
        long,
        short,
        long_help = "Archiving the message to history. use --no-archive/-A to disable"
    )]
    pub archive: bool,

    #[serde(skip)]
    #[arg(long, short = 'A', overrides_with = "archive", hide = true)]
    pub no_archive: bool,

    /// The push interruption level
    #[arg(
        long,
        short,
        // value_enum,
        // hide_possible_values = true,
        ignore_case = true,
        long_help = "The push interruption level. Simple as --active, --time-sensitive (alias --instant), --passive"
    )]
    pub level: Option<Level>,

    #[serde(skip)]
    #[arg(long, hide = true)]
    pub active: bool,

    #[serde(skip)]
    #[arg(long, visible_alias = "instant", hide = true)]
    pub time_sensitive: bool,

    #[serde(skip)]
    #[arg(long, hide = true)]
    pub passive: bool,

    /// The group name in history messages
    #[arg(long, short)]
    pub group: Option<String>,

    /// The sound name or sound url will be played
    #[arg(long)]
    pub sound: Option<String>,

    #[serde(flatten)]
    #[command(flatten)]
    pub encryption: Encryption,
}

#[derive(Debug, Default, Deserialize, Parser)]
pub struct Encryption {
    /// Push the encrypted message to server
    #[serde(default)]
    #[arg(
        long,
        short,
        long_help = "Push the encrypted message to server. Use --no-encrypt/-E to disable"
    )]
    pub encrypt: bool,

    #[serde(skip)]
    #[arg(long, short = 'E', overrides_with = "encrypt", hide = true)]
    pub no_encrypt: bool,

    /// Encryption method, e.g. aes128ecb aes256cbc
    #[arg(
        long,
        short,
        // value_enum,
        default_value = "aes128cbc",
        hide_possible_values = true,
        ignore_case = true,
        long_help = "Encryption method. Can be aes128cbc aes192cbc aes256cbc aes128ecb aes192ecb aes256ecb"
    )]
    pub method: Option<Method>,

    /// The AES key for encryption
    #[arg(long)]
    pub aes_key: Option<String>,

    /// The AES initialization vector for encryption
    #[arg(long)]
    pub aes_iv: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
pub enum Level {
    Active,
    TimeSensitive,
    Instant,
    Passive,
}

impl<'de> Deserialize<'de> for Level {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LevelVisitor;

        impl<'de> de::Visitor<'de> for LevelVisitor {
            type Value = Level;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing MyEnum")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "active" => Ok(Level::Active),
                    "instant" => Ok(Level::Instant),
                    "timesensitive" | "time_sensitive" | "time-sensitive" => {
                        Ok(Level::TimeSensitive)
                    }
                    "passive" => Ok(Level::Passive),
                    _ => Err(de::Error::unknown_variant(
                        value,
                        &["variant1", "variant2", "variant3"],
                    )),
                }
            }
        }
        deserializer.deserialize_str(LevelVisitor)
    }
}

impl Cli {
    pub fn to_msg(&self) -> Msg<'_> {
        Msg {
            body: self.body.as_str(),
            title: self.title.as_deref(),
            copy: self.copy.as_deref(),
            auto_copy: if self.auto_copy { Some(()) } else { None },
            archive: if self.conf.no_archive {
                Some(false)
            } else {
                Some(self.conf.archive)
            },
            level: self.conf.level,
            group: self.conf.group.as_deref(),
            url: self.url.as_deref(),
            sound: self.conf.sound.as_deref(),
            call: self.call,
            badge: self.badge.as_deref(),
        }
    }

    pub fn to_message(&self) -> Result<String> {
        let message =
            json5::to_string(&self.to_msg()).or(Err(anyhow!("Failed to serialize message")))?;

        if self.conf.encryption.is_valid_encryption() {
            bark::msg::encrypt(
                &message,
                self.conf.encryption.aes_key.as_deref().unwrap(),
                self.conf.encryption.aes_iv.as_deref(),
                self.conf.encryption.method.unwrap_or(Method::Aes128Cbc),
            )
            .or(Err(anyhow!("Failed to encrypt message")))
        } else {
            Ok(message)
        }
    }
}

impl Conf {
    /// Ensure that the fields needed for serialisation are set up correctly
    pub fn ensure_serialisation(&mut self) {
        if self.level.is_none() {
            if self.time_sensitive {
                self.level = Some(Level::TimeSensitive);
            } else if self.active {
                self.level = Some(Level::Active);
            } else if self.passive {
                self.level = Some(Level::Passive);
            }
        }
    }

    pub fn merge(self, other: Self) -> Self {
        let encryption = if self.encryption.no_encrypt || self.encryption.is_valid() {
            self.encryption
        } else if other.encryption.is_valid() {
            other.encryption
        } else {
            self.encryption
        };

        Self {
            server: self.server.or(other.server),
            device_key: self.device_key.or(other.device_key),
            archive: self.archive || other.archive,
            no_archive: self.no_archive || other.no_archive,
            level: self.level.or(other.level),
            active: false,
            time_sensitive: false,
            passive: false,
            group: self.group.or(other.group),
            sound: self.sound.or(other.sound),
            encryption,
        }
    }
}

impl Encryption {
    pub fn is_encrypted(&self) -> bool {
        !self.no_encrypt && self.encrypt
    }

    pub fn is_valid_encryption(&self) -> bool {
        if !self.is_encrypted() {
            false
        } else {
            self.check_encryption().is_ok()
        }
    }

    /// Check if the cipher is valid
    pub fn is_valid(&self) -> bool {
        self.encrypt && self.aes_key.is_some()
    }

    /// Check if the cipher is correct
    pub fn check_encryption(&self) -> Result<(), &'static str> {
        if self.aes_key.is_none() {
            return Err("Missing aes_key");
        }
        bark::msg::is_valid_cipher(
            self.method.unwrap_or(Method::Aes128Cbc),
            self.aes_key.as_ref().unwrap(),
            self.aes_iv.as_deref(),
        )
    }
}
