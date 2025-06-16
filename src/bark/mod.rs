mod encrypt;

use clap::{ArgAction, Args, ValueEnum, value_parser};
use serde::{Deserialize, Serialize, Serializer, de};

pub use encrypt::Encryption;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    #[serde(flatten)]
    pub service: Service,

    #[serde(flatten)]
    pub encryption: Encryption,

    #[serde(flatten)]
    pub stored: Storable,
}

#[derive(Deserialize, Args, Debug)]
pub struct Service {
    /// The server address of bark api service, default is https://api.day.app
    #[arg(long, short = 's')]
    #[serde(default)]
    server: Option<String>,

    /// Device key to receive push
    #[arg(long, short = 'd')]
    #[serde(default)]
    device_key: Option<String>,

    /// A list of device key to receive push
    #[arg(long, short = 'D', action = ArgAction::Append)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    device_keys: Vec<String>,
}

impl Service {
    pub fn merge(&mut self, other: Self) {
        if self.server.is_none() {
            self.server = other.server;
        }

        if self.device_key.is_none() {
            self.device_key = other.device_key;
        }

        self.device_keys.extend(other.device_keys);
        self.device_keys.dedup();
        if let Some(key) = self.device_key.as_ref() {
            if self.device_keys.contains(key) {
                self.device_key = None;
            }
        }
    }

    pub fn server(&self) -> &str {
        self.server.as_deref().unwrap_or(crate::API_SERVER)
    }

    pub fn device_keys(&self) -> Vec<&str> {
        let mut keys = self
            .device_keys
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>();
        keys.extend(self.device_key.as_deref());
        keys
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
pub enum Level {
    Critical,
    Active,
    #[clap(alias = "instant")]
    TimeSensitive,
    // Instant,
    Passive,
}

#[derive(Serialize, Deserialize, Args, Debug)]
pub struct Storable {
    /// Set different ringtones
    #[arg(long, short = 'S')]
    #[serde(skip_serializing_if = "Option::is_none")]
    sound: Option<String>,

    /// Set custom icons
    #[arg(long, short = 'I')]
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,

    /// Group messages
    #[arg(long, short = 'g')]
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,

    /// Pass "1" Save push, pass other push without saving
    #[arg(skip)]
    #[serde(rename = "isArchive", alias = "archive")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "ser_option_1"
    )]
    is_archive: Option<bool>,
}

#[derive(Serialize, Args, Debug)]
pub struct Push {
    /// Push title
    #[arg(long, short = 't')]
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    /// Push subtitle
    #[arg(long, short = 'T')]
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<String>,

    /// Push content
    #[arg(long, short = 'b')]
    body: String,

    /// Push interrupt level
    #[arg(long, short = 'l')]
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<Level>,

    /// Important warning notification volume
    #[arg(long, short = 'v', value_parser = value_parser!(u32).range(0..=10))]
    #[serde(skip_serializing_if = "Option::is_none")]
    volume: Option<u32>,

    /// Push angle marker, can be any number
    #[arg(long, short = 'B')]
    #[serde(skip_serializing_if = "Option::is_none")]
    badge: Option<u32>,

    /// Repeat notification ringtone
    #[arg(long, short = 'R')]
    #[serde(serialize_with = "ser_true_to_1", skip_serializing_if = "is_false")]
    call: bool,

    /// Automatically copy push content
    #[arg(long, short = 'C')]
    #[serde(rename = "autoCopy")]
    #[serde(serialize_with = "ser_true_to_1", skip_serializing_if = "is_false")]
    auto_copy: bool,

    /// Specify the copied content. If you do not pass this parameter, the entire push content will be copied.
    #[arg(long, short = 'c')]
    #[serde(skip_serializing_if = "Option::is_none")]
    copy: Option<String>,

    /// The URL that jumps when clicking push
    #[arg(long, short = 'u')]
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,

    /// When "none" is transmitted, clicking push will not pop up
    #[arg(long, overrides_with = "no_action")]
    #[serde(skip_serializing_if = "is_false", serialize_with = "ser_action")]
    action: bool,

    #[command(flatten)]
    #[serde(flatten)]
    store: Storable,
}

impl Storable {
    pub fn merge(&mut self, other: Self) {
        if self.sound.is_none() {
            self.sound = other.sound;
        }
        if self.icon.is_none() {
            self.icon = other.icon;
        }
        if self.group.is_none() {
            self.group = other.group;
        }
        if self.is_archive.is_none() {
            self.is_archive = other.is_archive;
        }
    }
}

impl Push {
    pub fn update_storable(&mut self, other: Storable) {
        self.store.merge(other)
    }

    pub fn update_level(&mut self, level: Option<Level>) {
        if level.is_some() {
            self.level = level;
        }
    }

    pub fn update_archive(&mut self, archive: Option<bool>) {
        if archive.is_some() {
            self.store.is_archive = archive;
        }
    }
}

fn ser_true_to_1<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *value {
        serializer.serialize_u32(1)
    } else {
        panic!()
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn ser_option_1<S>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(b) = value {
        if *b {
            serializer.serialize_u32(1)
        } else {
            serializer.serialize_u32(0)
        }
    } else {
        panic!()
    }
}

fn ser_action<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *value {
        serializer.serialize_str("none")
    } else {
        panic!()
    }
}

impl<'de> Deserialize<'de> for Level {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
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
                    "critical" => Ok(Level::Critical),
                    "active" => Ok(Level::Active),
                    // "instant" => Ok(Level::Instant),
                    "instant" | "timesensitive" | "time_sensitive" | "time-sensitive" => {
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
