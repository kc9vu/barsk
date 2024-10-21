use crate::command::Level;
use serde::Serializer;

// Skip serialize if value is false
pub fn is_false(value: &bool) -> bool {
    // Negate
    !value
}

pub fn serialize_archive<S>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.unwrap() {
        serializer.serialize_str("1")
    } else {
        serializer.serialize_str("0")
    }
}

pub fn serialize_level<S>(value: &Option<Level>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value.as_ref().unwrap() {
        Level::Active => serializer.serialize_str("active"),
        Level::TimeSensitive | Level::Instant => serializer.serialize_str("timeSensitive"),
        Level::Passive => serializer.serialize_str("passive"),
    }
}

pub fn serialize_call<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *value {
        serializer.serialize_str("1")
    } else {
        panic!("serialize_call called with false value")
    }
}
