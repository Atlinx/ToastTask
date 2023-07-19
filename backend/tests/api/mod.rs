pub mod auth;
pub mod crud_macros;
pub mod general;
pub mod list;

pub mod utils {
    use serde::Deserialize;
    use uuid::Uuid;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    pub struct GetAllResponse<T> {
        pub items: Vec<T>,
        pub limit: Option<u16>,
        pub offset: Option<u16>,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    pub struct PostResponse {
        pub id: Uuid,
    }
}
