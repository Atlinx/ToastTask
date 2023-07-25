use ::serde::{Deserialize, Deserializer, Serialize, Serializer};
use once_cell::sync::Lazy;
use std::collections::HashMap;
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

pub fn escape_sql_string(string: &str) -> String {
    let mut final_str = String::from("E'");
    static ESCAPED_CHARS_MAP: Lazy<HashMap<char, &str>> = Lazy::new(|| {
        HashMap::from([
            ('\\', r#"\\"#),
            ('\'', r#"\'"#),
            ('"', r#"\""#),
            ('\n', r#"\n"#),
            ('\t', r#"\t"#),
            ('\r', r#"\r"#),
        ])
    });

    for c in string.chars() {
        if let Some(escape_str) = ESCAPED_CHARS_MAP.get(&c) {
            final_str += escape_str;
        } else {
            final_str.push(c);
        }
    }
    final_str.push('\'');
    final_str
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
            match String -> String {
                crate::api::utils::escape_sql_string(&x)
            }
            match &String -> String {
                crate::api::utils::escape_sql_string(x)
            }
            match &str -> String {
                crate::api::utils::escape_sql_string(x)
            }
            match &&str -> String {
                crate::api::utils::escape_sql_string(*x)
            }
            match Uuid -> String {
                crate::api::utils::escape_sql_string(&x.to_string())
            }
            match &Uuid -> String {
                crate::api::utils::escape_sql_string(&x.to_string())
            }
            match &Option<String> -> String {
                match x {
                    Some(v) => crate::api::utils::escape_sql_string(&v),
                    None => String::from("NULL")
                }
            }
            match Option<String> -> String {
                match x {
                    Some(v) => crate::api::utils::escape_sql_string(&v),
                    None => String::from("NULL")
                }
            }
            match &Option<&str> -> String {
                match x {
                    Some(v) => crate::api::utils::escape_sql_string(v),
                    None => String::from("NULL")
                }
            }
            match Option<&str> -> String {
                match x {
                    Some(v) => crate::api::utils::escape_sql_string(v),
                    None => String::from("NULL")
                }
            }
            match &Option<Uuid> -> String {
                match x {
                    Some(v) => crate::api::utils::escape_sql_string(&v.to_string()),
                    None => String::from("NULL")
                }
            }
            match Option<Uuid> -> String {
                match x {
                    Some(v) => crate::api::utils::escape_sql_string(&v.to_string()),
                    None => String::from("NULL")
                }
            }
            match<T: std::string::ToString> &Option<T> -> String {
                match x {
                    Some(v) => v.to_string(),
                    None => String::from("NULL")
                }
            }
            match<T: std::string::ToString> Option<T> -> String {
                match x {
                    Some(v) => v.to_string(),
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

/// Macro that generates a SQL update query
///
/// Supports both a single update and batch updating
#[macro_export]
macro_rules! insert_query {
    // Single update
    ($table:expr; $($name:ident: $value:expr),*) => {
        crate::insert_query!($table; $($name: $value),*; "")
    };
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
    };

    // Batch updating
    ($table:expr; ($($name:ident: $index:tt),*); $value_tuples:expr) => {
        crate::insert_query! ($table; ($($name: $index),*); $value_tuples; "")
    };
    ($table:expr; ($($name:ident: $index:tt),*); $value_tuples:expr; $additional_sql:expr) => {
        {
            let names = vec![$(stringify!($name)),*];
            let mut values = Vec::<String>::new();

            for value_tuple in $value_tuples {

                let value_columns = Vec::<String>::new();
                $(
                    let column = value_tuple.$index;
                    let use_column = spez::spez! {
                        for x = &column;
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
                    if use_column {
                        value_columns.push(crate::print_sql!(column));
                    } else {
                        value_columns.push(String::from("default"));
                    }
                )*

                values.push(format!("({})", value_columns.join(", ")));
            }

            format!("INSERT INTO {} ({}) VALUES {} {}", $table, names.join(", "), values.join(", "), $additional_sql)
        }
    }
}

/// Macro that generates a SQL update query
#[macro_export]
macro_rules! update_query {
    ($table:expr; $($name:ident: $value:expr),*) => {
        crate::update_query!($table, $($name, $value),*; "")
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
