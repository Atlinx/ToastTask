#[macro_export]
macro_rules! api_relation_get {
  (
    model_table: $model_table:expr,
    model_singular: $model_singular:expr,
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

            static CHILD_ITEM_ID_STRING: &str = concat!("child_", stringify!($model_singular), "_id");
            static PARENT_ITEM_ID_STRING: &str = concat!("parent_", stringify!($model_singular), "_id");
            static QUERY_STRING: Lazy<String> = Lazy::new(|| {
                let get_fields = vec!($(stringify!($get_field+)),+);
                format!(r#"
                    SELECT {get_fields}, child_{singular}_id as "child_{singular}_id?", parent_{singular}_id as "parent_{singular}_id?" 
                        FROM {singular}_relations 
                        INNER JOIN (SELECT * FROM {table} WHERE user_id = $1 LIMIT $2 OFFSET $3) AS page_items
                        ON id = child_{singular}_id OR id = parent_{singular}_id"#,
                    get_fields = get_fields.join(","),
                    singular = $model_singular,
                    table = $model_table
                )
            });
            let query = sqlx::query(&QUERY_STRING)
            .bind(auth_user.id)
            .bind(limit as i64)
            .bind((page * limit) as i64)
            .fetch_all(&mut *db)
            .await
            .map_internal_server_error("Error fetching lists")?;

            let mut lists: HashMap<Uuid, GetModel> = HashMap::new();

            for row in query {
                let list = lists.entry(row.get("id")).or_insert_with(|| GetModel {
                    $($get_field: row.get(stringify!($get_field)),)+
                    parent: None,
                    children: Vec::new(),
                });
                if let (Some(parent_list_id), Some(child_list_id)) = (row.get(PARENT_ITEM_ID_STRING), row.get(CHILD_ITEM_ID_STRING))
                {
                    if list.id == parent_list_id {
                        list.children.push(child_list_id);
                    } else {
                        list.parent = Some(parent_list_id);
                    }
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
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            id: uuid::Uuid,
        ) -> crate::responses::APIResult {
            use crate::responses::{not_found, MapAPIResponse, APIResponse};
            use rocket::http::Status;
            use sqlx::Row;
            use once_cell::sync::Lazy;

            static CHILD_ITEM_ID_STRING: &str = concat!("child_", stringify!($model_singular), "_id");
            static PARENT_ITEM_ID_STRING: &str = concat!("parent_", stringify!($model_singular), "_id");
            static QUERY_STRING: Lazy<String> = Lazy::new(|| {
                let get_fields = vec!($(stringify!($get_field+)),+);
                format!(r#"
                    SELECT {get_fields}, child_{singular}_id as "child_{singular}_id?", parent_{singular}_id as "parent_{singular}_id?" 
                        FROM {singular}_relations 
                        INNER JOIN (SELECT * FROM {table} WHERE id = $1 AND user_id = $2) AS page_items
                        ON id = child_{singular}_id OR id = parent_{singular}_id"#,
                    get_fields = get_fields.join(","),
                    singular = $model_singular,
                    table = $model_table
                )
            });
            let query = sqlx::query(&QUERY_STRING)
            .bind(id)
            .bind(auth_user.id)
            .fetch_all(&mut *db)
            .await
            .map_internal_server_error("Error fetching lists")?;

            let mut maybe_list: Option<GetModel> = None;
            for row in query {
                let list = maybe_list.get_or_insert_with(|| GetModel {
                    $($get_field: row.get(stringify!($get_field)),)+
                    parent: None,
                    children: Vec::new(),
                });
                if let (Some(parent_list_id), Some(child_list_id)) = (row.get(PARENT_ITEM_ID_STRING), row.get(CHILD_ITEM_ID_STRING))
                {
                    if list.id == parent_list_id {
                        list.children.push(child_list_id);
                    } else {
                        list.parent = Some(parent_list_id);
                    }
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
macro_rules! api_relation_post {
    (
        model_table: $model_table:expr,
        input: $input:path,
        input_fields: { $($input_field:ident),+ },
        create_item_relation: $create_item_relation:path
    ) => {
        #[post("/", data = "<input>", format = "application/json")]
        async fn post(
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            input: rocket_validation::Validated<rocket::serde::json::Json<$input>>,
        ) -> crate::responses::APIResult {
            use crate::{
                api::utils::PostResponse,
                responses::{MapAPIResponse, APIResponse}
            };
            use sqlx::{Acquire, Row};
            use rocket::http::Status;
            use once_cell::sync::Lazy;

            let input = input.0;
            let mut trans = db
                .begin()
                .await
                .map_internal_server_error("Create item transaction failed to start.")?;
            static QUERY_STRING: Lazy<String> = Lazy::new(|| {
                let fields = vec![$(stringify!($input_field)),+];
                let value_fields: Vec<String> = fields.iter().enumerate().map(|(idx, _)| format!("${}", idx + 2)).collect();
                format!("INSERT INTO {} (user_id, {}) VALUES ($1, {}) RETURNING id", $model_table, fields.join(", "), value_fields.join(", "))
            });
            let created = sqlx::query(&QUERY_STRING)
                .bind(auth_user.id)
                $(.bind(input.$input_field.clone()))+
                .fetch_one(&mut trans).await;
            let created = created.map_internal_server_error("Failed to create in database.")?;

            if let Some(parent_id) = input.parent {
                $create_item_relation(&mut trans, created.get("id"), parent_id).await?;
            }

            trans
                .commit()
                .await
                .map_internal_server_error("Failed to commit create list transaction")?;

            let resp = PostResponse { id: created.get("id") };
            Ok(APIResponse::new(
                Status::Created,
                serde_json::to_value(resp)
                    .map_internal_server_error("Failed to convert response into json.")?,
            ))
        }
    }
}

#[macro_export]
macro_rules! api_relation_patch {
    (
        model_table: $model_table:expr,
        input: $input:path,
        input_fields: { $($name:ident),+ },
        delete_child_item_relation: $delete_child_item_relation:path,
        create_item_relation: $create_item_relation:path
    ) => {
        #[patch("/<id>", data = "<input>", format = "application/json")]
        async fn patch(
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            input: rocket_validation::Validated<rocket::serde::json::Json<$input>>,
            id: uuid::Uuid,
        ) -> crate::responses::APIResult {
            use crate::responses::{
                bad_request, internal_server_error, ok, result_bad_request, result_not_found,
                MapAPIResponse,
            };
            use sqlx::Acquire;

            let input = input.0;
            let mut trans = db
                .begin()
                .await
                .map_internal_server_error("Patch list transaction failed to start.")?;
            let update_str = crate::update_set! {
                $model_table;
                $($name: input.$name),+;
                "WHERE id = $1 AND user_id = $2"
            };
            if let Some(update_str) = update_str {
                let result = sqlx::query(&update_str)
                    .bind(id)
                    .bind(auth_user.id)
                    .execute(&mut trans)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::Database(e) => {
                            bad_request(&format!("Invalid patch request. {}", e))
                        }
                        _ => internal_server_error("Failed to patch in database."),
                    })?;
                if result.rows_affected() == 0 {
                    return result_not_found("Item not found.");
                }
            }
            match input.parent {
                Patch::Null => {
                    $delete_child_item_relation(&mut trans, id).await?;
                }
                Patch::Value(parent_id) => {
                    $delete_child_item_relation(&mut trans, id).await?;

                    if id == parent_id {
                        return result_bad_request("Cannot make list a parent of itself.");
                    }

                    $create_item_relation(&mut trans, id, parent_id).await?;
                }
                Patch::Missing => (),
            }
            trans
                .commit()
                .await
                .map_internal_server_error("Failed to commit patch list transaction.")?;

            Ok(ok("Patch successful."))
        }
    };
}

#[macro_export]
macro_rules! api_relation_delete {
    (
        model_table: $model_table:expr
    ) => {
        #[delete("/<id>")]
        async fn delete(
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            id: uuid::Uuid,
        ) -> crate::responses::APIResult {
            use crate::responses::{ok, result_not_found, MapAPIResponse};

            static QUERY_STRING: Lazy<String> = Lazy::new(|| {
                format!(
                    "DELETE FROM {} WHERE id = $1 AND user_id = $2",
                    stringify!($model_table)
                )
            });
            let res = sqlx::query(&QUERY_STRING)
                .bind(id)
                .bind(auth_user.id)
                .execute(&mut *db)
                .await
                .map_internal_server_error("Failed to delete in database.")?;
            if res.rows_affected() == 0 {
                return result_not_found("Item not found.");
            }
            Ok(ok("Delete successful."))
        }
    };
}

#[macro_export]
macro_rules! api_relation_utils {
    (
        model_singular: $model_singular:expr
    ) => {
        mod relation_utils {
            use crate::responses::{
                bad_request, internal_server_error, APIResponse, MapAPIResponse,
            };
            use once_cell::sync::Lazy;
            use sqlx::Executor;

            pub async fn delete_child_item_relation<'q, E>(
                executor: E,
                child_id: uuid::Uuid,
            ) -> Result<(), APIResponse>
            where
                E: Executor<'q, Database = sqlx::Postgres>,
            {
                static QUERY_STRING: Lazy<String> = Lazy::new(|| {
                    format!(
                        "DELETE FROM {0}_relations WHERE child_{0}_id = $1",
                        $model_singular
                    )
                });
                sqlx::query(&QUERY_STRING)
                    .bind(child_id)
                    .execute(executor)
                    .await
                    .map_internal_server_error("Failed to delete old list relation.")?;
                Ok(())
            }

            pub async fn create_item_relation<'q, E>(
                executor: E,
                child_id: uuid::Uuid,
                parent_id: uuid::Uuid,
            ) -> Result<(), APIResponse>
            where
                E: Executor<'q, Database = sqlx::Postgres>,
            {
                sqlx::query(&format!(
                    "INSERT INTO {0}_relations (child_{0}_id, parent_{0}_id) VALUES ($1, $2)",
                    $model_singular
                ))
                .bind(child_id)
                .bind(parent_id)
                .execute(executor)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(db_e) if db_e.code().unwrap_or_default() == "23503" => {
                        // 23503 = Foreign key error code
                        //         https://www.postgresql.org/docs/current/errcodes-appendix.html
                        bad_request("Invalid parent id.")
                    }
                    _ => internal_server_error("Failed to create parent relationship in database."),
                })?;
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! api_relation_crud {
    (
        model_table: $model_table:expr,
        model_singular: $model_singular:expr,
        model_fields: { $($model_field:ident),+ },
        get_model: $get_model:path,
        post_input: $post_input:path,
        patch_input: $patch_input:path
    ) => {
        crate::api_relation_crud!(
            model_table: $model_table,
            model_singular: $model_singular,
            get: {
                model_type: $get_model,
                get_fields: id, user_id, $($model_field),+
            },
            post: {
                input: $post_input,
                input_fields: $($model_field),+
            },
            patch: {
                input: $patch_input,
                input_fields: $($model_field),+
            },
            delete: {}
        );
    };
    (
        model_table: $model_table:expr,
        model_singular: $model_singular:expr,
        get: {
            model_type: $get_model:path,
            get_fields: $($get_field:ident),+
        },
        post: {
            input: $post_input:path,
            input_fields: $($post_input_field:ident),+
        },
        patch: {
            input: $patch_input:path,
            input_fields: $($patch_input_field:ident),+
        },
        delete: {}
    ) => {
        crate::api_relation_get! {
            model_table: $model_table,
            model_singular: $model_singular,
            model_type: $get_model,
            get_fields: { $($get_field),+ }
        }
        crate::api_relation_post! {
            model_table: $model_table,
            input: $post_input,
            input_fields: { $($post_input_field),+ },
            create_item_relation: relation_utils::create_item_relation
        }
        crate::api_relation_patch! {
            model_table: $model_table,
            input: $patch_input,
            input_fields: { $($patch_input_field),+ },
            delete_child_item_relation: relation_utils::delete_child_item_relation,
            create_item_relation: relation_utils::create_item_relation
        }
        crate::api_delete! {
            model_table: $model_table
        }
        crate::api_relation_utils! {
            model_singular: $model_singular
        }

        pub fn mount_rocket(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
            rocket.mount(format!("/{}", $model_table), routes![get_all, get_single, post, patch, delete])
        }
    }
}
