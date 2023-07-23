#[macro_export]
macro_rules! api_tree_get {
  (
    model_table: $model_table:expr,
    model_type: $model:path,
    get_fields: { $($get_field:ident),+ }
  ) => {
        #[get("/?<limit>&<page>")]
        async fn get_all(
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            limit: Option<u32>,
            page: Option<u32>,
        ) -> crate::responses::APIResult {
            use crate::{
                api::utils::{GET_LIMIT, GetAllResponse},
                responses::{MapAPIResponse, APIResponse}
            };
            use std::{
                collections::HashMap,
            };
            use rocket::http::Status;
            use sqlx::Row;
            use $model as GetModel;
            use once_cell::sync::Lazy;

            let limit = limit.unwrap_or(GET_LIMIT);
            let page = page.unwrap_or(0);

            static QUERY_STRING: Lazy<String> = Lazy::new(|| {
                let get_fields = vec!($(concat!("base_query.", stringify!($get_field))),+);
                format!(r#"
                    SELECT {get_fields}, child_query.id AS child_id FROM 
                        (SELECT * FROM {table} WHERE user_id = $1 LIMIT $2 OFFSET $3) base_query
                        LEFT JOIN {table} child_query
                        ON base_query.id = child_query.parent_id"#,
                    get_fields = get_fields.join(", "),
                    table = $model_table
                )
            });
            let query = sqlx::query(&QUERY_STRING)
            .bind(auth_user.id)
            .bind(limit as i64)
            .bind((page * limit) as i64)
            .fetch_all(&mut *db)
            .await
            .map_internal_server_error("Error fetching items")?;

            let mut items: HashMap<Uuid, GetModel> = HashMap::new();

            for row in query {
                let list = items.entry(row.get("id")).or_insert_with(|| GetModel {
                    $($get_field: row.get(stringify!($get_field)),)+
                    child_ids: Vec::new(),
                });
                if let Some(child_id) = row.get("child_id") {
                    list.child_ids.push(child_id);
                }
            }

            let resp = GetAllResponse::<GetModel> {
                items: items.into_values().collect(),
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
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            id: uuid::Uuid,
        ) -> crate::responses::APIResult {
            use crate::responses::{not_found, MapAPIResponse, APIResponse};
            use rocket::http::Status;
            use sqlx::Row;
            use once_cell::sync::Lazy;

            static QUERY_STRING: Lazy<String> = Lazy::new(|| {
                let get_fields = vec!($(concat!("base_query.", stringify!($get_field))),+);
                format!(r#"
                    SELECT {get_fields}, child_query.id AS child_id FROM 
                        (SELECT * FROM {table} WHERE id = $1 AND user_id = $2) base_query
                        LEFT JOIN {table} child_query
                        ON base_query.id = child_query.parent_id"#,
                    get_fields = get_fields.join(", "),
                    table = $model_table
                )
            });
            let query = sqlx::query(&QUERY_STRING)
            .bind(id)
            .bind(auth_user.id)
            .fetch_all(&mut *db)
            .await
            .map_internal_server_error("Error fetching items")?;

            let mut maybe_list: Option<GetModel> = None;
            for row in query {
                let list = maybe_list.get_or_insert_with(|| GetModel {
                    $($get_field: row.get(stringify!($get_field)),)+
                    child_ids: Vec::new(),
                });
                if let Some(child_id) = row.get("child_id") {
                    list.child_ids.push(child_id);
                }
            }
            let list = maybe_list.ok_or_else(|| not_found("Item not found."))?;

            Ok(APIResponse::new(
                Status::Ok,
                serde_json::to_value(list)
                    .map_internal_server_error("Failed to convert response into json.")?,
            ))
        }
    }
}

#[macro_export]
macro_rules! api_tree_crud {
    (
        model_table: $model_table:expr,
        model_fields: { $($model_field:ident),+ },
        get_model: $get_model:path,
        post_input: $post_input:path,
        patch_input: $patch_input:path
    ) => {
        crate::api_tree_crud!(
            model_table: $model_table,
            get: {
                model_type: $get_model,
                get_fields: { id, user_id, $($model_field),+ }
            },
            post: {
                input: $post_input,
                input_fields: { $($model_field),+ }
            },
            patch: {
                input: $patch_input,
                input_fields: { $($model_field),+ }
            },
            delete: {}
        );
    };
    (
        model_table: $model_table:expr,
        get: {
            model_type: $get_model:path,
            get_fields: { $($get_field:ident),+ }
        },
        post: {
            input: $post_input:path,
            input_fields: { $($post_input_field:ident),+ }
        },
        patch: {
            input: $patch_input:path,
            input_fields: { $($patch_input_field:ident),+ }
        },
        delete: {}
    ) => {
        crate::api_tree_get! {
            model_table: $model_table,
            model_type: $get_model,
            get_fields: { $($get_field),+ }
        }
        crate::api_post! {
            model_table: $model_table,
            input: $post_input,
            input_fields: { $($post_input_field),+ }
        }
        crate::api_patch! {
            model_table: $model_table,
            input: $patch_input,
            input_fields: { $($patch_input_field),+ }
        }
        crate::api_delete! {
            model_table: $model_table
        }

        pub fn mount_rocket(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
            rocket.mount(format!("/{}", $model_table), routes![get_all, get_single, post, patch, delete])
        }
    }
}
