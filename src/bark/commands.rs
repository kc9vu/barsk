use clap::ValueEnum;
use serde::{
    de::{Error, Visitor},
    Deserialize, Serialize,
};
use Level::*;

#[derive(ValueEnum, Clone, Serialize, Debug)]
pub(crate) enum Level {
    Active,
    TimeSensitive,
    Instant,
    Passive,
}

impl From<Level> for String {
    fn from(level: Level) -> Self {
        match level {
            Active => "active".to_string(),
            TimeSensitive | Instant => "timeSensitive".to_string(),
            Passive => "passive".to_string(),
        }
    }
}

impl<'a> From<&Level> for &'a str {
    fn from(level: &Level) -> Self {
        match level {
            Active => "active",
            TimeSensitive | Instant => "timeSensitive",
            Passive => "passive",
        }
    }
}

impl<'de> Deserialize<'de> for Level {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LevelVisitor;

        impl<'de> Visitor<'de> for LevelVisitor {
            type Value = Level;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing MyEnum")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value.to_lowercase().as_str() {
                    "active" => Ok(Level::Active),
                    "instant" => Ok(Level::Instant),
                    "timesensitive" | "time_sensitive" | "time-sensitive" => {
                        Ok(Level::TimeSensitive)
                    }
                    "passive" => Ok(Level::Passive),
                    _ => Err(Error::unknown_variant(
                        value,
                        &["variant1", "variant2", "variant3"],
                    )),
                }
            }
        }
        deserializer.deserialize_str(LevelVisitor)
    }
}
