pub mod config;
pub mod structs;

use clap::{ArgGroup, Parser};
use serde::{Serialize, Serializer};

use cipher::{encrypt, Cipher};
use config::Conf;
use structs::Level;

#[derive(Parser, Serialize, Debug)]
#[command(
    name = "barsk",
    author,
    about,
    version,
    max_term_width = 80,
    group(
        ArgGroup::new("push level")
            .args(["active", "time_sensitive", "passive"])
            .conflicts_with("level")
    ),
    long_about = "",
)]
pub struct BarkArgs {
    #[arg(long, short, help = "")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[arg(long, short, help = "")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy: Option<String>,

    #[arg(long, short = 'C', help = "")]
    #[serde(rename = "autoCopy", skip_serializing_if = "is_false")]
    pub auto_copy: bool,

    #[arg(long, short, help = "")]
    #[serde(
        rename = "isArchive",
        skip_serializing_if = "is_false",
        serialize_with = "serialize_archive"
    )]
    pub archive: bool,

    #[arg(long, overrides_with = "archive", hide = true)]
    #[serde(
        rename = "isArchive",
        skip_serializing_if = "is_false",
        serialize_with = "serialize_no_archive"
    )]
    pub no_archive: bool,

    #[arg(
        long,
        short,
        value_enum,
        hide_possible_values = true,
        ignore_case = true,
        help = "Level of the push",
        long_help = "Level of the push. Simple as --active, --time-sensitive (alias --instant), --passive"
    )]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_level"
    )]
    pub level: Option<Level>,

    #[arg(long, hide = true)]
    #[serde(skip)]
    pub active: bool,

    #[arg(long, visible_alias = "instant", hide = true)]
    #[serde(skip)]
    pub time_sensitive: bool,

    #[arg(long, hide = true)]
    #[serde(skip)]
    pub passive: bool,

    #[arg(long, short, help = "")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    #[arg(long, help = "")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[arg(long, help = "")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,

    #[arg(long, help = "")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    #[arg(long, short, help = "")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,

    #[arg(long, short, help = "")]
    #[serde(skip)]
    pub encrypt: bool,

    #[arg(long, short = 'E' , overrides_with = "encrypt", hide = true, help = "")]
    #[serde(skip)]
    pub no_encrypt: bool,

    #[arg(
        long,
        value_enum,
        hide_possible_values = true,
        ignore_case = true,
        help = "Encryption detail, e.g. aes128-cbc aes256-cbc"
    )]
    #[serde(skip)]
    pub cipher: Option<Cipher>,

    #[arg(long, help = "")]
    #[serde(skip)]
    pub key: Option<String>,

    #[arg(long, help = "")]
    #[serde(skip)]
    pub iv: Option<String>,

    #[arg(long, short, help = "")]
    #[serde(skip)]
    pub server: Option<String>,

    #[arg(long, short, help = "")]
    #[serde(skip)]
    pub device_key: Option<String>,

    #[arg(long = "config", short = 'F', help = "")]
    #[serde(skip)]
    pub config_file: Option<String>,

    #[arg(long, short = 'z', help = "")]
    #[serde(skip)]
    pub thats_all: bool,

    #[arg(long, short = 'p', help = "")]
    #[serde(skip)]
    pub dry_run: bool,

    #[arg(help = "Push content")]
    pub body: String,
}

impl BarkArgs {
    pub fn parse() -> Result<Self, String> {
        let mut args = <BarkArgs as Parser>::parse();

        if args.active {
            args.level = Some(Level::Active);
        } else if args.time_sensitive {
            args.level = Some(Level::TimeSensitive);
        } else if args.passive {
            args.level = Some(Level::Passive);
        }

        if !args.thats_all {
            if args.config_file.is_some() {
                args.update(Conf::file(args.config_file.as_ref().unwrap())?)
            } else if let Some(conf) = Conf::default_file()? {
                args.update(conf);
            };
        }
        if args.server.as_deref().is_some_and(|v| !v.contains("://")) {
            args.server = Some(format!("https://{}", args.server.unwrap()));
        }

        Ok(args)
    }

    pub fn check(&self) -> Result<(), String> {
        if self.server.is_none() || self.device_key.is_none() {
            return Err("Missing server or device_id".into());
        }
        self.check_cipher()?;
        Ok(())
    }

    pub fn check_cipher(&self) -> Result<(), String> {
        if self.encrypt {
            if let Some(cipher) = self.cipher {
                match cipher {
                    Cipher::Aes128Cbc | Cipher::Aes192Cbc | Cipher::Aes256Cbc
                        if (self.key.is_none() || self.iv.is_none()) =>
                    {
                        Err("Key and(or) iv should be provided".into())
                    }
                    Cipher::Aes128Ecb | Cipher::Aes192Ecb | Cipher::Aes256Ecb
                        if self.key.is_none() =>
                    {
                        Err("Key should be provided".into())
                    }
                    _ => Ok(()),
                }
            } else {
                Err("No cipher specified".into())
            }
        } else {
            Ok(())
        }
    }

    pub fn to_message(&self) -> Result<String, String> {
        let msg = serde_json::to_string(self).unwrap();

        if self.encrypt {
            Ok(encrypt(
                &msg,
                self.key.as_ref().unwrap(),
                self.iv.as_deref(),
                self.cipher.unwrap(),
            )?)
        } else {
            Ok(msg)
        }
    }

    pub fn update(&mut self, conf: Conf) {
        macro_rules! update_field {
            ($field:ident) => {
                if self.$field.is_none() && conf.$field.is_some() {
                    self.$field = conf.$field;
                }
            };
        }

        update_field!(server);
        update_field!(device_key);
        update_field!(level);
        update_field!(group);
        update_field!(icon);
        update_field!(sound);

        if !(self.archive || self.no_archive) {
            match conf.archive {
                Some(true) => self.archive = true,
                Some(false) => self.no_archive = true,
                None => {}
            }
        }
        if !(self.encrypt || self.no_encrypt) {
            self.encrypt = conf.encrypt.unwrap_or(false);
        }
        if self.encrypt {
            update_field!(cipher);
            if self.key.is_none() && self.iv.is_none() {
                self.key = conf.key;
                self.iv = conf.iv;
            }
        }
    }

    pub fn print(&self) -> Result<(), String> {
        println!(
            "The message will be sent to {}/{}\n{}",
            self.server.as_deref().unwrap_or("no_server"),
            if self.device_key.is_some() {
                "xxxxxx"
            } else {
                "no_device_key"
            },
            &self.to_message()?,
        );
        Ok(())
    }
}

/// Skip serialize if value is false
fn is_false(value: &bool) -> bool {
    // true when value is false
    !value
}

fn serialize_archive<S>(_value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("1")
}
fn serialize_no_archive<S>(_value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("0")
}
fn serialize_level<S>(value: &Option<Level>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value.unwrap() {
        Level::Active => serializer.serialize_str("active"),
        Level::TimeSensitive | Level::Instant => serializer.serialize_str("timeSensitive"),
        Level::Passive => serializer.serialize_str("passive"),
    }
}
