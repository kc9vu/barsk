use serde::{
    de::{Error, Visitor},
    Deserialize, Serialize,
};

#[derive(clap::ValueEnum, Clone, Copy, Serialize, Debug)]
pub enum Level {
    Active,
    TimeSensitive,
    Instant,
    Passive,
}

#[derive(Deserialize)]
pub struct Res {
    pub code: u16,
    pub message: String,
    pub timestamp: u64,
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
