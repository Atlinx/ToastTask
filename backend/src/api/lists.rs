use once_cell::sync::Lazy;
use regex::Regex;
use rocket::{http::Status, serde::json::Json, Build, Rocket};
use rocket_db_pools::Connection;
use rocket_validation::Validated;
use serde::{Deserialize, Serialize};
use sqlx::Error::RowNotFound;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::{
    database::BackendDb,
    guards::auth::Auth,
    models::{list::ListModel, user::UserModel},
    responses::{
        internal_server_error, not_found, ok, result_not_found, APIResponse, APIResult,
        MapAPIResponse,
    },
    update_set,
};

use super::utils::{GetAllResponse, Patch, PostResponse, GET_LIMIT};

#[get("/?<limit>&<page>")]
async fn get_all(
    auth_user: Auth<UserModel>,
    mut db: Connection<BackendDb>,
    limit: Option<u32>,
    page: Option<u32>,
) -> Result<APIResponse, APIResponse> {
    let limit = limit.unwrap_or(GET_LIMIT);
    let page = page.unwrap_or(0);

    let items = sqlx::query_as!(
        ListModel,
        "SELECT * FROM lists WHERE user_id = $1 LIMIT $2 OFFSET $3",
        auth_user.id,
        limit as i64,
        (page * limit) as i64
    )
    .fetch_all(&mut *db)
    .await
    .map_internal_server_error("Error fetching lists")?;

    let resp = GetAllResponse::<ListModel> { items, limit, page };

    Ok(APIResponse::new(
        Status::Ok,
        serde_json::to_value(resp)
            .map_internal_server_error("Failed to convert response into json.")?,
    ))
}

#[get("/<id>")]
async fn get_single(
    auth_user: Auth<UserModel>,
    mut db: Connection<BackendDb>,
    id: Uuid,
) -> APIResult {
    let item = sqlx::query_as!(
        ListModel,
        "SELECT * FROM lists WHERE id = $1 AND user_id = $2",
        id,
        auth_user.id
    )
    .fetch_one(&mut *db)
    .await
    .map_err(|e| match e {
        RowNotFound => not_found("Item not found."),
        _ => internal_server_error("Error fetching lists"),
    })?;

    Ok(APIResponse::new(
        Status::Ok,
        serde_json::to_value(item)
            .map_internal_server_error("Failed to convert response into json.")?,
    ))
}

#[post("/", data = "<input>", format = "application/json")]
async fn post(
    auth_user: Auth<UserModel>,
    mut db: Connection<BackendDb>,
    input: Validated<Json<PostInput>>,
) -> APIResult {
    let input = input.0;
    let created = sqlx::query!(
        "INSERT INTO lists(user_id, title, description, color) VALUES ($1, $2, $3, $4) RETURNING id",
        auth_user.id,
        input.title,
        input.description,
        input.color
    ).fetch_one(&mut *db).await;
    let created = created.map_internal_server_error("Failed to create in database.")?;
    let resp = PostResponse { id: created.id };
    Ok(APIResponse::new(
        Status::Created,
        serde_json::to_value(resp)
            .map_internal_server_error("Failed to convert response into json.")?,
    ))
}

#[patch("/", data = "<input>", format = "application/json")]
async fn patch(
    auth_user: Auth<UserModel>,
    mut db: Connection<BackendDb>,
    input: Validated<Json<PatchInput>>,
) -> APIResult {
    let input = input.0;
    let result = sqlx::query(update_set! {
        "lists";
        title: input.title,
        description: input.description,
        color: input.color;
        "WHERE user_id = $1"
    })
    .bind(auth_user.id)
    .execute(&mut *db)
    .await
    .map_internal_server_error("Failed to patch in database.")?;
    if result.rows_affected() == 0 {
        return result_not_found("Item not found.");
    }
    Ok(ok("Patch successful."))
}

#[delete("/<id>")]
async fn delete(auth_user: Auth<UserModel>, mut db: Connection<BackendDb>, id: Uuid) -> APIResult {
    let res = sqlx::query!(
        "DELETE FROM lists WHERE id = $1 AND user_id = $2",
        id,
        auth_user.id
    )
    .execute(&mut *db)
    .await
    .map_internal_server_error("Failed to delete in database.")?;
    if res.rows_affected() == 0 {
        return result_not_found("Item not found.");
    }
    Ok(ok("Delete successful."))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PatchInput {
    #[serde(default)]
    pub title: Patch<String>,
    #[serde(default)]
    pub description: Patch<String>,
    #[serde(default)]
    #[validate(custom = "validate_patch_color")]
    pub color: Patch<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PostInput {
    pub title: String,
    pub description: Option<String>,
    #[validate(custom = "validate_color")]
    pub color: String,
}

fn validate_patch_color(color: &Patch<String>) -> Result<(), ValidationError> {
    println!("Validated patch color: {:#?}", color);
    match color {
        Patch::Missing | Patch::Null => Ok(()),
        Patch::Value(ref color_str) => validate_color(color_str),
    }
}

fn validate_color(color: &str) -> Result<(), ValidationError> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#([A-Fa-f0-9]{6})$").unwrap());
    if REGEX.is_match(color) {
        return Ok(());
    }
    Err(ValidationError::new(
        "Color must follow the 6 digit hex format (#ffffff).",
    ))
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/lists", routes![get_all, get_single, post, patch, delete])
}
