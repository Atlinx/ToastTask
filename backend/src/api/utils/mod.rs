use ::serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

pub mod crud_macros;
pub mod serde;
pub mod tree_crud_macros;
pub mod validation;

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
        S: Serializer,
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
    ($e:expr; no_recurse) => {
        spez::spez! {
            for x = $e;
            match String -> String {
                format!("'{}'", x)
            }
            match &String -> String {
                format!("'{}'", x)
            }
            match &str -> String {
                format!("'{}'", x)
            }
            match &&str -> String {
                format!("'{}'", x)
            }
            match Uuid -> String {
                format!("'{}'", x)
            }
            match &Uuid -> String {
                format!("'{}'", x)
            }
            match<T: std::string::ToString> T -> String {
                x.to_string()
            }
            match<T: std::string::ToString> &T -> String {
                x.to_string()
            }
            match<T> T -> String {
                String::from("NULL")
            }
        }
    };
    ($e:expr) => {
        spez::spez! {
            for x = $e;
            match String -> String {
                format!("'{}'", x)
            }
            match &String -> String {
                format!("'{}'", x)
            }
            match &str -> String {
                format!("'{}'", x)
            }
            match &&str -> String {
                format!("'{}'", x)
            }
            match Uuid -> String {
                format!("'{}'", x)
            }
            match &Uuid -> String {
                format!("'{}'", x)
            }
            match<T: std::string::ToString> &Option<T> -> String {
                match x {
                    Some(v) => {
                        crate::print_sql!(v; no_recurse)
                    }
                    None => String::from("NULL")
                }
            }
            match<T: std::string::ToString> Option<T> -> String {
                match x {
                    Some(v) => {
                        crate::print_sql!(v; no_recurse)
                    }
                    None => String::from("NULL")
                }
            }
            match<T: std::string::ToString> T -> String {
                x.to_string()
            }
            match<T: std::string::ToString> &T -> String {
                x.to_string()
            }
            match<T> T -> String {
                String::from("NULL")
            }
        }
    };
}

#[macro_export]
macro_rules! post_query {
    ($table:expr; $($name:ident: $value:expr),*; $additional_sql:expr) => {
        {
            let mut names = Vec::<&str>::new();
            let mut values = Vec::<String>::new();

            $(
                let use_value = spez::spez! {
                    for x = &$value;
                    match<T> &Option<T> -> bool {
                        match x {
                            Some(_) => true,
                            None => false
                        }
                    }
                    match<T> &T -> bool {
                        true
                    }
                };
                if use_value {
                    names.push(stringify!($name));
                    values.push(crate::print_sql!($value));
                }
            )*

            format!("INSERT INTO {} ({}) VALUES ({}) {}", $table, names.join(", "), values.join(", "), $additional_sql)
        }
    }
}

#[macro_export]
macro_rules! update_set {
    ($table:expr; $($name:ident: $value:expr),*) => {
        update_set!($table, $($name, $value),*; "")
    };
    ($table:expr; $($name:ident: $value:expr),*; $additional_sql:expr) => {
        {
            use crate::api::utils::{Patch};

            let mut changes_exist = false;
            let mut name_value_pairs: Vec<String> = Vec::new();
            $(
                match &$value {
                    Patch::Missing => (),
                    Patch::Null => {
                        changes_exist = true;
                        name_value_pairs.push(concat!( stringify!($name), "=NULL").to_owned())
                    },
                    Patch::Value(ref real_value) => {
                        changes_exist = true;
                        name_value_pairs.push(format!("{}={}", stringify!($name), crate::print_sql!(real_value)))
                    }
                }
            )*

            if changes_exist {
                Some(format!("UPDATE {} SET {} {}", $table, name_value_pairs.join(", "), $additional_sql))
            } else {
                None
            }
        }
    }
}
