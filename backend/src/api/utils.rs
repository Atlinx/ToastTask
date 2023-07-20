use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

pub const GET_LIMIT: u32 = 1000;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetAllResponse<T> {
    pub items: Vec<T>,
    pub limit: u32,
    pub page: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostResponse {
    pub id: Uuid,
}

#[derive(Debug)]
pub enum Patch<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Patch::Missing
    }
}

impl<T> From<Option<T>> for Patch<T> {
    fn from(opt: Option<T>) -> Patch<T> {
        match opt {
            Some(v) => Patch::Value(v),
            None => Patch::Null,
        }
    }
}

impl<'de, T> Deserialize<'de> for Patch<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Into::into)
    }
}

impl<T> Serialize for Patch<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Missing => serializer.serialize_none(),
            Self::Null => serializer.serialize_none(),
            Self::Value(value) => serializer.serialize_some(value),
        }
    }
}

/// Macro to print any value in SQL.
///
/// This had to be a macro because rust doesn't support specialization yet,
/// which would allow us to implement a default behaviour for all T: Display,
/// and then implement a more specific behaviour for String.
#[macro_export]
macro_rules! print_sql {
    ($e:expr) => {
        spez::spez! {
            for x = $e;
            match &String -> String {
                format!("'{}'", x)
            }
            match &&str -> String {
                format!("'{}'", x)
            }
            match<T: std::fmt::Display> T -> String {
                format!("{}", x)
            }
            match<T> T -> String {
                String::from("NULL")
            }
        }
    };
}

#[macro_export]
macro_rules! update_set {
    ($table:expr; $($name:ident: $value:expr),*) => {
        update_set!($table, $($name, $value),*; "")
    };
    ($table:expr; $($name:ident: $value:expr),*; $additional_sql:expr) => {
        {
            use crate::api::utils::{Patch};

            let mut name_value_pairs: Vec<String> = Vec::new();
            $(
                match &$value {
                    Patch::Missing => (),
                    Patch::Null => name_value_pairs.push(concat!( stringify!($name), "=NULL").to_owned()),
                    Patch::Value(ref real_value) => name_value_pairs.push(format!("{}={}", stringify!($name), crate::print_sql!(real_value)))
                }
            )*

            &format!("UPDATE {} SET {} {}", $table, name_value_pairs.join(", "), $additional_sql)
        }
    }
}
