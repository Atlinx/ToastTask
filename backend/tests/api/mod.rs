pub mod auth;
pub mod general;
pub mod list;

pub mod utils {
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    pub struct GetAllResponse<T> {
        pub items: Vec<T>,
        pub limit: Option<u16>,
        pub offset: Option<u16>,
    }
}
