use crate::bark::Level;
use serde::Serializer;

// Skip serialize if value is false
pub(crate) fn is_false(value: &bool) -> bool {
    // Negate
    !value
}

#[allow(unused)]
pub(crate) fn is_some_true(value: &Option<bool>) -> bool {
    matches!(value, Some(true))
}

#[allow(unused)]
pub(crate) fn is_some_false(value: &Option<bool>) -> bool {
    matches!(value, Some(false))
}

#[allow(unused)]
pub(crate) fn serialize_archive<S>(_value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("1")
}

#[allow(unused)]
pub(crate) fn serialize_no_archive<S>(_value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str("0")
}

#[allow(unused)]
pub(crate) fn serialize_level<S>(value: &Option<Level>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value.as_ref().unwrap() {
        Level::Active => serializer.serialize_str("active"),
        Level::TimeSensitive | Level::Instant => serializer.serialize_str("timeSensitive"),
        Level::Passive => serializer.serialize_str("passive"),
    }
}

pub(crate) fn serialize_call<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *value {
        serializer.serialize_str("1")
    } else {
        panic!("serialize_call called with false value")
    }
}
