use rocket::{data, http::Status, outcome::Outcome, Request};
use serde::Serialize;
use serde_json::Value;

use crate::responses::APIResponse;

pub trait ToGuardOutcome<T>
where
    Self: Sized,
{
    fn to_guard_outcome<'a>(self, req: &'a Request<'_>) -> data::Outcome<'a, T, Value> {
        self.to_guard_outcome_status(req, Status::BadRequest)
    }
    fn to_guard_outcome_status<'a>(
        self,
        req: &'a Request<'_>,
        status: Status,
    ) -> data::Outcome<'a, T, Value>;
}

impl<T, E> ToGuardOutcome<T> for Result<T, E>
where
    E: Serialize,
{
    fn to_guard_outcome_status<'a>(
        self,
        req: &'a Request<'_>,
        status: Status,
    ) -> data::Outcome<'a, T, Value> {
        match self {
            Ok(r) => Outcome::Success(r),
            Err(e) => Outcome::Failure(match serde_json::to_value(e) {
                Ok(error_json) => APIResponse::new(status, error_json)
                    .cache_guard_error(req)
                    .into(),
                Err(_) => APIResponse::new_message(
                    Status::InternalServerError,
                    "Failed to validate input.",
                )
                .cache_guard_error(req)
                .into(),
            }),
        }
    }
}
