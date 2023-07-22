use std::collections::HashMap;

use super::utils::{Patch, PostResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use rocket::{http::Status, serde::json::Json, Build, Rocket};
use rocket_db_pools::Connection;
use rocket_validation::Validated;
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{self, PgDatabaseError},
    Acquire,
};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::{
    api::utils::{GetAllResponse, GET_LIMIT},
    responses::{bad_request, ok, result_not_found, APIResponse, MapAPIResponse},
};
use crate::{
    database::BackendDb,
    guards::auth::Auth,
    models::user::UserModel,
    responses::{internal_server_error, not_found, APIResult},
};

#[get("/?<limit>&<page>")]
async fn get_all(
    auth_user: Auth<UserModel>,
    mut db: rocket_db_pools::Connection<BackendDb>,
    limit: Option<u32>,
    page: Option<u32>,
) -> APIResult {
    let limit = limit.unwrap_or(GET_LIMIT);
    let page = page.unwrap_or(0);

    let query = sqlx::query!(
        "SELECT * FROM list_relations 
            INNER JOIN (SELECT * FROM lists WHERE user_id = $1 LIMIT $2 OFFSET $3) AS page_lists
            ON id = child_list_id OR id = parent_list_id",
        auth_user.id,
        limit as i64,
        (page * limit) as i64
    )
    .fetch_all(&mut *db)
    .await
    .map_internal_server_error("Error fetching lists")?;

    let mut lists: HashMap<Uuid, GetModel> = HashMap::new();

    for row in query {
        let list = lists.entry(row.id).or_insert_with(|| GetModel {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            description: row.description,
            color: row.color,
            parent: None,
            children: Vec::new(),
        });
        if list.id == row.parent_list_id {
            list.children.push(row.child_list_id);
        } else if list.id == row.child_list_id {
            list.parent = Some(row.child_list_id);
        }
    }

    let resp = GetAllResponse::<GetModel> {
        items: lists.into_values().collect(),
        limit,
        page,
    };

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
    let query = sqlx::query!(
        "SELECT * FROM list_relations 
            INNER JOIN (SELECT * FROM lists WHERE id = $1 AND user_id = $2) as page_lists
            ON id = child_list_id OR id = parent_list_id",
        id,
        auth_user.id
    )
    .fetch_all(&mut *db)
    .await
    .map_internal_server_error("Error fetching lists")?;

    let mut maybe_list: Option<GetModel> = None;
    for row in query {
        let list = maybe_list.get_or_insert_with(|| GetModel {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            description: row.description,
            color: row.color,
            parent: None,
            children: Vec::new(),
        });
        if list.id == row.parent_list_id {
            list.children.push(row.child_list_id);
        } else if list.id == row.child_list_id {
            list.parent = Some(row.child_list_id);
        }
    }

    let list = maybe_list.ok_or_else(|| not_found("Item not found."))?;
    Ok(APIResponse::new(
        Status::Ok,
        serde_json::to_value(list)
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

    let mut trans = db
        .begin()
        .await
        .map_internal_server_error("Create list transaction failed to start.")?;
    let created = sqlx::query!("INSERT INTO lists (user_id, title, description, color) VALUES ($1, $2, $3, $4) RETURNING id", auth_user.id, input.title, input.description, input.color)
        .fetch_one(&mut trans).await;
    let created = created.map_internal_server_error("Failed to create in database.")?;

    if let Some(parent_id) = input.parent {
        sqlx::query!(
            "INSERT INTO list_relations (child_list_id, parent_list_id) VALUES ($1, $2)",
            created.id,
            parent_id
        )
        .execute(&mut trans)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_e) if db_e.code().unwrap_or_default() == "23503" => {
                // 23503 = Foreign key error code
                //         https://www.postgresql.org/docs/current/errcodes-appendix.html
                bad_request("Invalid parent id.")
            }
            _ => internal_server_error("Failed to create parent relationship in database."),
        })?;
    }

    trans
        .commit()
        .await
        .map_internal_server_error("Failed to commit create list transaction")?;

    let resp = PostResponse { id: created.id };
    Ok(APIResponse::new(
        Status::Created,
        serde_json::to_value(resp)
            .map_internal_server_error("Failed to convert response into json.")?,
    ))
}

#[patch("/<id>", data = "<input>", format = "application/json")]
async fn patch(
    auth_user: Auth<UserModel>,
    mut db: Connection<BackendDb>,
    input: Validated<Json<PatchInput>>,
    id: Uuid,
) -> APIResult {
    let input = input.0;
    let update_str = crate::update_set! {
        "lists";
        title: input.title,
        description: input.description,
        color: input.color;
        "WHERE id = $1 AND user_id = $2"
    };

    let mut trans = db
        .begin()
        .await
        .map_internal_server_error("Patch list transaction failed to start.")?;
    let result = sqlx::query(update_str)
        .bind(id)
        .bind(auth_user.id)
        .execute(&mut trans)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(_) => bad_request("Invalid patch request."),
            _ => internal_server_error("Failed to patch in database."),
        })?;
    if result.rows_affected() == 0 {
        return result_not_found("Item not found.");
    }
    match input.parent {
        Patch::Null => {
            sqlx::query!("DELETE FROM list_relations WHERE child_list_id = $1", id)
                .execute(&mut trans)
                .await
                .map_internal_server_error("Failed to delete old list relation.")?;
        }
        Patch::Value(parent_id) => {
            sqlx::query!("DELETE FROM list_relations WHERE child_list_id = $1", id)
                .execute(&mut trans)
                .await
                .map_internal_server_error("Failed to delete old list relation.")?;
            sqlx::query!(
                "INSERT INTO list_relations(child_list_id, parent_list_id) VALUES ($1, $2)",
                id,
                parent_id
            )
            .execute(&mut trans)
            .await
            .map_internal_server_error("Failed to insert new list relation.")?;
        }
        Patch::Missing => (),
    }
    trans
        .commit()
        .await
        .map_internal_server_error("Failed to commit patch list transaction.")?;
    Ok(ok("Patch successful."))
}

crate::api_delete! {
    model_table: "lists"
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
    pub parent: Patch<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PostInput {
    pub title: String,
    pub description: Option<String>,
    #[validate(custom = "validate_color")]
    pub color: String,
    pub parent: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub color: String,
    pub parent: Option<Uuid>,
    pub children: Vec<Uuid>,
}

fn validate_patch_color(color: &Patch<String>) -> Result<(), ValidationError> {
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
    rocket.mount("/lists", routes![get_single, get_all, post, patch, delete])
}
