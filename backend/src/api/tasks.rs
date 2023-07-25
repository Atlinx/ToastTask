use once_cell::sync::Lazy;
use rocket::{http::Status, serde::json::Json, Build, Rocket};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use time::PrimitiveDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{
    api::utils::{serde::primitive_date_iso_serialize, GetAllResponse, GET_LIMIT},
    database::BackendDb,
    guards::auth::Auth,
    models::user::UserModel,
    responses::{
        internal_server_error, not_found, ok, result_not_found, APIResponse, APIResult,
        MapAPIResponse,
    },
};

use super::utils::Patch;

#[get("/?<limit>&<page>")]
async fn get_all(
    auth_user: Auth<UserModel>,
    mut db: Connection<BackendDb>,
    limit: Option<u32>,
    page: Option<u32>,
) -> APIResult {
    let limit = limit.unwrap_or(GET_LIMIT);
    let page = page.unwrap_or(0);

    let query: Vec<_> = sqlx::query!(
        r#"
        SELECT 
            base_tasks.*,
            child_tasks.id AS "child_id?",
            task_labels.label_id as "label_id?"
            FROM (
                SELECT tasks.* FROM tasks
                    INNER JOIN lists
                    ON lists.id = tasks.list_id
                WHERE user_id = $1 LIMIT $2 OFFSET $3
            ) base_tasks
            LEFT JOIN tasks child_tasks 
                ON base_tasks.id = child_tasks.parent_id
            LEFT JOIN task_labels 
                ON base_tasks.id = task_labels.task_id"#,
        auth_user.id,
        limit as i64,
        (page * limit) as i64
    )
    .fetch_all(&mut *db)
    .await
    .map_internal_server_error("Error fetching items")?;

    let mut items: HashMap<Uuid, GetModelBuilder> = HashMap::new();
    for row in query {
        let item = items.entry(row.id).or_insert_with(|| GetModelBuilder {
            get_model: GetModel {
                id: row.id,
                parent_id: row.parent_id,
                list_id: row.list_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                due_at: row.due_at,
                due_text: row.due_text,
                completed: row.completed,
                title: row.title,
                description: row.description,
                child_ids: vec![],
                label_ids: vec![],
            },
            child_ids: HashSet::new(),
            label_ids: HashSet::new(),
        });
        if let Some(child_id) = row.child_id {
            item.child_ids.insert(child_id);
        }
        if let Some(label_id) = row.label_id {
            item.label_ids.insert(label_id);
        }
    }

    let items = items.into_values().map(|row| row.build()).collect();
    let resp = GetAllResponse::<GetModel> { items, limit, page };

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
        r#"
        SELECT 
            base_tasks.*,
            child_tasks.id AS "child_id?",
            task_labels.label_id as "label_id?"
            FROM (
                SELECT tasks.* FROM tasks
                    INNER JOIN lists
                    ON lists.id = tasks.list_id
                WHERE tasks.id = $1 AND user_id = $2
            ) base_tasks
            LEFT JOIN tasks child_tasks 
                ON base_tasks.id = child_tasks.parent_id
            LEFT JOIN task_labels 
                ON base_tasks.id = task_labels.task_id"#,
        id,
        auth_user.id
    )
    .fetch_all(&mut *db)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => not_found("Item not found."),
        _ => internal_server_error("Error fetching items"),
    })?;

    let mut maybe_task: Option<GetModelBuilder> = None;
    for row in query {
        let task = maybe_task.get_or_insert_with(|| GetModelBuilder {
            get_model: GetModel {
                id: row.id,
                parent_id: row.parent_id,
                list_id: row.list_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                due_at: row.due_at,
                due_text: row.due_text,
                completed: row.completed,
                title: row.title,
                description: row.description,
                child_ids: vec![],
                label_ids: vec![],
            },
            child_ids: HashSet::new(),
            label_ids: HashSet::new(),
        });
        if let Some(child_id) = row.child_id {
            task.child_ids.insert(child_id);
        }
        if let Some(label_id) = row.label_id {
            task.label_ids.insert(label_id);
        }
    }
    let item = maybe_task.ok_or_else(|| not_found("Item not found"))?;

    Ok(APIResponse::new(
        Status::Ok,
        serde_json::to_value(item.build())
            .map_internal_server_error("Failed to convert response into json.")?,
    ))
}

crate::api_patch! {
    model_table: "tasks",
    input: PatchInput,
    input_fields: { parent_id, list_id, due_at, due_text, completed, title, description },
    query_where: "WHERE id = $1 AND list_id IN (SELECT id FROM lists WHERE user_id = $2)"
    // Only let the user patch tasks they own.
}

crate::api_post! {
    model_table: "tasks",
    input: PostInput,
    input_fields: { parent_id, list_id, due_at, due_text, completed, title, description },
    user_id: false
}

crate::api_delete! {
    model_table: "tasks",
    query_where: "WHERE id = $1 AND list_id IN (SELECT id FROM lists WHERE user_id = $2)"
    // Only let the user delete tasks they own.
}

#[post("/<id>/labels", data = "<input>", format = "application/json")]
async fn post_label(
    auth_user: Auth<UserModel>,
    input: Json<LabelPostInput>,
    mut db: Connection<BackendDb>,
    id: Uuid,
) -> APIResult {
    let res = sqlx::query!(
        "SELECT tasks.id, lists.user_id FROM tasks 
            INNER JOIN lists ON tasks.list_id = lists.id
            WHERE tasks.id = $1 AND lists.user_id = $2
        ",
        id,
        auth_user.id
    )
    .fetch_optional(&mut *db)
    .await
    .map_internal_server_error("Failed to fetch task from database.")?;

    if let None = res {
        return result_not_found("Task not found");
    }

    sqlx::query!(
        "INSERT INTO task_labels(task_id, label_id) VALUES ($1, $2)",
        id,
        input.id
    )
    .execute(&mut *db)
    .await
    .map_internal_server_error("Failed to attach label in database.")?;

    Ok(ok("Label attached successfully."))
}

#[delete("/<id>/labels/<label_id>")]
async fn delete_label(
    auth_user: Auth<UserModel>,
    mut db: Connection<BackendDb>,
    id: Uuid,
    label_id: Uuid,
) -> APIResult {
    sqlx::query!(
        "DELETE FROM task_labels 
            WHERE label_id = $1 AND 
                task_id = $2 AND
                label_id IN (SELECT id FROM labels WHERE user_id = $3)",
        label_id,
        id,
        auth_user.id
    )
    .execute(&mut *db)
    .await
    .map_internal_server_error("Failed to detach label in database.")?;

    Ok(ok("Label detached successfully."))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PatchInput {
    #[serde(default)]
    pub parent_id: Patch<Uuid>,
    #[serde(default)]
    pub list_id: Patch<Uuid>,
    #[serde(default)]
    pub due_at: Patch<String>,
    #[serde(default)]
    pub due_text: Patch<String>,
    #[serde(default)]
    pub completed: Patch<bool>,
    #[serde(default)]
    pub title: Patch<String>,
    #[serde(default)]
    pub description: Patch<String>,
    #[serde(default)]
    pub label_ids: Patch<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PostInput {
    pub parent_id: Option<Uuid>,
    pub list_id: Uuid,
    pub due_at: String,
    pub due_text: String,
    pub completed: Option<bool>,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetModel {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub list_id: Uuid,
    #[serde(serialize_with = "primitive_date_iso_serialize")]
    pub created_at: PrimitiveDateTime,
    #[serde(serialize_with = "primitive_date_iso_serialize")]
    pub updated_at: PrimitiveDateTime,
    #[serde(serialize_with = "primitive_date_iso_serialize")]
    pub due_at: PrimitiveDateTime,
    pub due_text: String,
    pub completed: bool,
    pub title: String,
    pub description: Option<String>,
    pub child_ids: Vec<Uuid>,
    pub label_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelPostInput {
    pub id: Uuid,
}

#[derive(Debug)]
struct GetModelBuilder {
    pub get_model: GetModel,
    pub child_ids: HashSet<Uuid>,
    pub label_ids: HashSet<Uuid>,
}

impl GetModelBuilder {
    pub fn build(self) -> GetModel {
        let mut get_model = self.get_model;
        get_model.child_ids = self.child_ids.into_iter().collect();
        get_model.label_ids = self.label_ids.into_iter().collect();
        get_model
    }
}

pub fn mount_rocket(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount(
        "/tasks",
        routes![
            get_single,
            get_all,
            post,
            patch,
            delete,
            post_label,
            delete_label
        ],
    )
}
