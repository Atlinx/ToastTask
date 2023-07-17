use rocket::http::{ContentType, Status};
use rocket::request::local_cache;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::serde::json::{json, Value};
use std::convert::From;
use std::io::Cursor;

pub type RequestGuardError = (Status, Value);

/// Response to an API call. All
/// responses store their data in JSON,
/// and have a status code.
#[derive(Debug, Clone)]
pub struct APIResponse {
    /// Data of the response in JSON
    data: Value,
    /// Status of the response
    status: Status,
}

pub struct CachedAPIResponse(pub APIResponse);

impl APIResponse {
    /// Creates an API response from a status and a mesasge.
    /// The message will be presented as
    ///
    /// ```json
    /// {
    ///   "message": "my api response message"
    /// }
    /// ```
    pub fn new_message(status: Status, message: &str) -> APIResponse {
        APIResponse {
            data: json!({ "message": message }),
            status,
        }
    }

    /// Creates an API response from a status with some JSON data.
    pub fn new(status: Status, data: Value) -> APIResponse {
        APIResponse { data, status }
    }

    /// Set the data of the `Response` to `data`.
    pub fn data(mut self, data: Value) -> APIResponse {
        self.data = data;
        self
    }

    /// Convenience method to set `self.data` to `{"message": message}`.
    pub fn message(mut self, message: &str) -> APIResponse {
        self.data = json!({ "message": message });
        self
    }

    pub fn as_guard_error(&self) -> RequestGuardError {
        (self.status, self.data.clone())
    }

    pub fn cache_guard_error(self, req: &Request) -> APIResponse {
        req.local_cache(|| CachedAPIResponse(self.clone()));
        self
    }
}

impl From<Status> for APIResponse {
    /// Creates an API response from a status.
    ///
    /// `Status::Ok`, `Status::Created`, `Status::Accepted`,
    /// and `Status::NoContent` return a response with only
    /// a status code.
    ///
    /// The remaining stuses return their status code along with
    /// the response name as part of the body.
    ///
    /// Ex.
    /// ```rust
    /// from_status(Status::BadGateway);
    /// ```
    /// returns a response with the body
    /// ```json
    /// {
    ///   "message": "Bad Gateway"
    /// }
    /// ```
    fn from(status: Status) -> Self {
        if status.code == Status::Ok.code
            || status.code == Status::Created.code
            || status.code == Status::Accepted.code
            || status.code == Status::NoContent.code
        {
            return APIResponse::new(status, json!(null));
        }
        APIResponse::new_message(status, &status.to_string())
    }
}

impl From<sqlx::Error> for APIResponse {
    fn from(_: sqlx::Error) -> Self {
        APIResponse::from(Status::InternalServerError)
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for APIResponse {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let json_body_str = self.data.to_string();
        Response::build()
            .status(self.status)
            .sized_body(json_body_str.len(), Cursor::new(json_body_str))
            .header(ContentType::JSON)
            .ok()
    }
}

impl From<APIResponse> for RequestGuardError {
    fn from(value: APIResponse) -> Self {
        value.as_guard_error()
    }
}
