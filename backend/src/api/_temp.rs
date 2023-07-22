// use std::collections::HashMap;
// use super::utils::{Patch, PostResponse};
// use once_cell::sync::Lazy;
// use regex::Regex;
// use rocket::{http::Status, serde::json::Json, Build, Rocket};
// use rocket_db_pools::Connection;
// use rocket_validation::Validated;
// use serde::{Deserialize, Serialize};
// use sqlx::{Acquire, Database, Execute, Executor};
// use uuid::Uuid;
// use validator::{Validate, ValidationError};

// use crate::{
//     api::utils::{GetAllResponse, GET_LIMIT},
//     responses::{
//         bad_request, ok, result_bad_request, result_not_found, APIResponse, MapAPIResponse,
//     },
// };
// use crate::{
//     database::BackendDb,
//     guards::auth::Auth,
//     models::user::UserModel,
//     responses::{internal_server_error, not_found, APIResult},
// };

// #[get("/?<limit>&<page>")]
// async fn get_all(
//     auth_user: Auth<UserModel>,
//     mut db: rocket_db_pools::Connection<BackendDb>,
//     limit: Option<u32>,
//     page: Option<u32>,
// ) -> APIResult {
//     let limit = limit.unwrap_or(GET_LIMIT);
//     let page = page.unwrap_or(0);

//     let query = sqlx::query!(
//         r#"SELECT id, user_id, title, description, color, child_list_id as "child_list_id?", parent_list_id as "parent_list_id?"
//             FROM list_relations
//             INNER JOIN (SELECT * FROM lists WHERE user_id = $1 LIMIT $2 OFFSET $3) AS page_lists
//             ON id = child_list_id OR id = parent_list_id"#,
//         auth_user.id,
//         limit as i64,
//         (page * limit) as i64
//     )
//     .fetch_all(&mut *db)
//     .await
//     .map_internal_server_error("Error fetching lists")?;

//     let mut lists: HashMap<Uuid, GetModel> = HashMap::new();

//     for row in query {
//         let list = lists.entry(row.id).or_insert_with(|| GetModel {
//             id: row.id,
//             user_id: row.user_id,
//             title: row.title,
//             description: row.description,
//             color: row.color,
//             parent: None,
//             children: Vec::new(),
//         });
//         if let (Some(parent_list_id), Some(child_list_id)) = (row.parent_list_id, row.child_list_id)
//         {
//             if list.id == parent_list_id {
//                 list.children.push(child_list_id);
//             } else {
//                 list.parent = Some(parent_list_id);
//             }
//         }
//     }

//     let resp = GetAllResponse::<GetModel> {
//         items: lists.into_values().collect(),
//         limit,
//         page,
//     };

//     Ok(APIResponse::new(
//         Status::Ok,
//         serde_json::to_value(resp)
//             .map_internal_server_error("Failed to convert response into json.")?,
//     ))
// }

// #[get("/<id>")]
// async fn get_single(
//     auth_user: Auth<UserModel>,
//     mut db: Connection<BackendDb>,
//     id: Uuid,
// ) -> APIResult {
//     let query = sqlx::query!(
//         r#"SELECT id, user_id, title, description, color, child_list_id as "child_list_id?", parent_list_id as "parent_list_id?"
//             FROM (SELECT * FROM lists WHERE id = $1 AND user_id = $2) as page_lists
//             LEFT JOIN list_relations
//             ON id = child_list_id OR id = parent_list_id"#,
//         id,
//         auth_user.id
//     )
//     .fetch_all(&mut *db)
//     .await
//     .map_internal_server_error("Error fetching lists")?;

//     let mut maybe_list: Option<GetModel> = None;
//     for row in query {
//         let list = maybe_list.get_or_insert_with(|| GetModel {
//             id: row.id,
//             user_id: row.user_id,
//             title: row.title,
//             description: row.description,
//             color: row.color,
//             parent: None,
//             children: Vec::new(),
//         });
//         if let (Some(parent_list_id), Some(child_list_id)) = (row.parent_list_id, row.child_list_id)
//         {
//             if list.id == parent_list_id {
//                 list.children.push(child_list_id);
//             } else {
//                 list.parent = Some(parent_list_id);
//             }
//         }
//     }
//     let list = maybe_list.ok_or_else(|| not_found("Item not found."))?;

//     Ok(APIResponse::new(
//         Status::Ok,
//         serde_json::to_value(list)
//             .map_internal_server_error("Failed to convert response into json.")?,
//     ))
// }

// #[post("/", data = "<input>", format = "application/json")]
// async fn post(
//     auth_user: Auth<UserModel>,
//     mut db: Connection<BackendDb>,
//     input: Validated<Json<PostInput>>,
// ) -> APIResult {
//     let input = input.0;

//     let mut trans = db
//         .begin()
//         .await
//         .map_internal_server_error("Create list transaction failed to start.")?;
//     let created = sqlx::query!("INSERT INTO lists (user_id, title, description, color) VALUES ($1, $2, $3, $4) RETURNING id", auth_user.id, input.title, input.description, input.color)
//         .fetch_one(&mut trans).await;
//     let created = created.map_internal_server_error("Failed to create in database.")?;

//     if let Some(parent_id) = input.parent {
//         create_list_relation(&mut trans, created.id, parent_id).await?;
//     }

//     trans
//         .commit()
//         .await
//         .map_internal_server_error("Failed to commit create list transaction")?;

//     let resp = PostResponse { id: created.id };
//     Ok(APIResponse::new(
//         Status::Created,
//         serde_json::to_value(resp)
//             .map_internal_server_error("Failed to convert response into json.")?,
//     ))
// }

// #[patch("/<id>", data = "<input>", format = "application/json")]
// async fn patch(
//     auth_user: Auth<UserModel>,
//     mut db: Connection<BackendDb>,
//     input: Validated<Json<PatchInput>>,
//     id: Uuid,
// ) -> APIResult {
//     let input = input.0;
//     let update_str = crate::update_set! {
//         "lists";
//         title: input.title,
//         description: input.description,
//         color: input.color;
//         "WHERE id = $1 AND user_id = $2"
//     };

//     let mut trans = db
//         .begin()
//         .await
//         .map_internal_server_error("Patch list transaction failed to start.")?;
//     if let Some(update_str) = update_str {
//         let result = sqlx::query(&update_str)
//             .bind(id)
//             .bind(auth_user.id)
//             .execute(&mut trans)
//             .await
//             .map_err(|e| match e {
//                 sqlx::Error::Database(e) => bad_request(&format!("Invalid patch request. {}", e)),
//                 _ => internal_server_error("Failed to patch in database."),
//             })?;
//         if result.rows_affected() == 0 {
//             return result_not_found("Item not found.");
//         }
//     }
//     match input.parent {
//         Patch::Null => {
//             delete_child_list_relation(&mut trans, id).await?;
//         }
//         Patch::Value(parent_id) => {
//             delete_child_list_relation(&mut trans, id).await?;

//             if id == parent_id {
//                 return result_bad_request("Cannot make list a parent of itself.");
//             }

//             create_list_relation(&mut trans, id, parent_id).await?;
//         }
//         Patch::Missing => (),
//     }
//     trans
//         .commit()
//         .await
//         .map_internal_server_error("Failed to commit patch list transaction.")?;

//     Ok(ok("Patch successful."))
// }

// crate::api_delete! {
//     model_table: "lists"
// }
