use serde::{Deserialize, Deserializer, Serializer};
use time::{format_description::well_known::Iso8601, OffsetDateTime, PrimitiveDateTime};

pub fn string_serialize<T, S>(x: &T, s: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    s.serialize_str(&x.to_string())
}

pub fn primitive_date_iso_serialize<S>(x: &PrimitiveDateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(
        &x.assume_utc()
            .format(&Iso8601::DEFAULT)
            .map_err(|_| serde::ser::Error::custom("Failed to serialize date time into iso."))?,
    )
}

pub fn primitive_date_iso_deserialize<'de, D>(
    deserializer: D,
) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let date_time_str: &str = Deserialize::deserialize(deserializer)?;
    PrimitiveDateTime::parse(date_time_str, &Iso8601::DEFAULT)
        .map_err(|_| serde::de::Error::custom("Failed to parse into date time."))
}
