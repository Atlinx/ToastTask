use rocket::http::{ContentType, Status};
use rocket::outcome::Outcome;
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

    pub fn as_cache_guard_error(self, req: &Request) -> RequestGuardError {
        req.local_cache(|| CachedAPIResponse(self.clone()));
        self.as_guard_error()
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
    /// use toast_task::responses::APIResponse;
    /// use rocket::http::Status;
    ///
    /// APIResponse::from(Status::BadGateway);
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

impl<'r, 'o: 'r> Responder<'r, 'o> for APIResponse {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'o> {
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

pub trait MapAPIResponse<R>
where
    Self: Sized,
{
    fn map_response(self, status: Status, data: Value) -> R;

    fn map_response_message(self, status: Status, message: &str) -> R;

    fn map_internal_server_error(self, message: &str) -> R {
        self.map_response_message(Status::InternalServerError, message)
    }

    fn map_bad_request(self, message: &str) -> R {
        self.map_response_message(Status::BadRequest, message)
    }

    fn map_unauthorized(self, message: &str) -> R {
        self.map_response_message(Status::Unauthorized, message)
    }
}

impl<T, E> MapAPIResponse<Result<T, APIResponse>> for Result<T, E> {
    fn map_response(self, status: Status, data: Value) -> Result<T, APIResponse> {
        self.map_err(|_| APIResponse::new(status, data))
    }

    fn map_response_message(self, status: Status, message: &str) -> Result<T, APIResponse> {
        self.map_err(|_| APIResponse::new_message(status, message))
    }
}

pub trait MapReqAPIResponse<R>
where
    Self: Sized,
{
    fn map_response(self, req: &Request<'_>, status: Status, data: Value) -> R;

    fn map_response_message(self, req: &Request<'_>, status: Status, message: &str) -> R;

    fn map_internal_server_error(self, req: &Request<'_>, message: &str) -> R {
        self.map_response_message(req, Status::InternalServerError, message)
    }

    fn map_bad_request(self, req: &Request<'_>, message: &str) -> R {
        self.map_response_message(req, Status::BadRequest, message)
    }

    fn map_unauthorized(self, req: &Request<'_>, message: &str) -> R {
        self.map_response_message(req, Status::Unauthorized, message)
    }
}

impl<S, E, F> MapReqAPIResponse<Outcome<S, RequestGuardError, F>> for Outcome<S, E, F> {
    fn map_response(
        self,
        req: &Request<'_>,
        status: Status,
        data: Value,
    ) -> Outcome<S, RequestGuardError, F> {
        self.map_failure(|_| APIResponse::new(status, data).as_cache_guard_error(req))
    }

    fn map_response_message(
        self,
        req: &Request<'_>,
        status: Status,
        message: &str,
    ) -> Outcome<S, RequestGuardError, F> {
        self.map_failure(|_| APIResponse::new_message(status, message).as_cache_guard_error(req))
    }
}

// Used as shorthands for message based API responses
pub fn internal_server_error(message: &str) -> APIResponse {
    APIResponse::new_message(Status::InternalServerError, message)
}

pub fn bad_request(message: &str) -> APIResponse {
    APIResponse::new_message(Status::BadRequest, message)
}

pub fn unauthorized(message: &str) -> APIResponse {
    APIResponse::new_message(Status::Unauthorized, message)
}

// Used within handlers to return an error for a Result<APIResponse, APIResponse>
pub fn result_internal_server_error(message: &str) -> Result<APIResponse, APIResponse> {
    Err(internal_server_error(message))
}

pub fn result_bad_request(message: &str) -> Result<APIResponse, APIResponse> {
    Err(bad_request(message))
}

pub fn result_unauthorized(message: &str) -> Result<APIResponse, APIResponse> {
    Err(unauthorized(message))
}

// Used within request guards to return a failure for an Outcome<_, RequestGuard, _>
pub fn guard_internal_server_error<S, F>(
    req: &Request<'_>,
    message: &str,
) -> Outcome<S, RequestGuardError, F> {
    Outcome::Failure(internal_server_error(message).as_cache_guard_error(req))
}

pub fn guard_bad_request<S, F>(
    req: &Request<'_>,
    message: &str,
) -> Outcome<S, RequestGuardError, F> {
    Outcome::Failure(bad_request(message).as_cache_guard_error(req))
}

pub fn guard_unauthorized<S, F>(
    req: &Request<'_>,
    message: &str,
) -> Outcome<S, RequestGuardError, F> {
    Outcome::Failure(unauthorized(message).as_cache_guard_error(req))
}
