#[macro_export]
macro_rules! api_get {
    (
        model_table: $model_table:expr,
        model_type: $model:path
    ) => {
        crate::api_get! {
            model_table: $model_table,
            model_type: $model,
            query_single_where: "WHERE id = $1 AND user_id = $2",
            query_all_where: "WHERE user_id = $1"
        }
    };
    (
        model_table: $model_table:expr,
        model_type: $model:path,
        query_single_where: $query_single_where:expr,
        query_all_where: $query_all_where:expr
    ) => {
        #[get("/?<limit>&<page>")]
        async fn get_all(
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            limit: Option<u32>,
            page: Option<u32>,
        ) -> crate::responses::APIResult {
            use once_cell::sync::Lazy;
            use rocket::http::Status;
            use $model as ItemModel;

            use crate::{
                api::utils::{GetAllResponse, GET_LIMIT},
                responses::{APIResponse, MapAPIResponse},
            };

            let limit = limit.unwrap_or(GET_LIMIT);
            let page = page.unwrap_or(0);

            static QUERY_STRING: Lazy<String> = Lazy::new(|| {
                format!(
                    "SELECT * FROM {} {} LIMIT $2 OFFSET $3",
                    $model_table, $query_all_where
                )
            });
            let items: Vec<ItemModel> = sqlx::query_as::<_, ItemModel>(&QUERY_STRING)
                .bind(auth_user.id)
                .bind(limit as i64)
                .bind((page * limit) as i64)
                .fetch_all(&mut *db)
                .await
                .map_internal_server_error("Error fetching lists")?;

            let resp = GetAllResponse::<ItemModel> { items, limit, page };

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
            use rocket::http::Status;
            use sqlx::Error::RowNotFound;
            use $model as ItemModel;

            use crate::responses::{internal_server_error, not_found, APIResponse, MapAPIResponse};

            static QUERY_STRING: Lazy<String> =
                Lazy::new(|| format!("SELECT * FROM {} {}", $model_table, $query_single_where));
            let item: ItemModel = sqlx::query_as::<_, ItemModel>(&QUERY_STRING)
                .bind(id)
                .bind(auth_user.id)
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
    };
}

#[macro_export]
macro_rules! api_post {
    (
        model_table: $model_table:expr,
        input: $input:path,
        input_fields: { $($input_field:ident),+ }
    ) => {
        crate::api_post! {
            model_table: $model_table,
            input: $input,
            input_fields: { $($input_field),+ },
            user_id: true
        }
    };
    (
        model_table: $model_table:expr,
        input: $input:path,
        input_fields: { $($input_field:ident),+ },
        user_id: $user_id:expr
    ) => {
        #[post("/", data = "<input>", format = "application/json")]
        async fn post(
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            input: rocket::serde::json::Json<$input>,
        ) -> crate::responses::APIResult {
            use rocket::http::Status;
            use sqlx::Row;
            use sqlx::postgres::PgRow;

            use crate::{
                responses::{APIResponse, MapAPIResponse},
                api::utils::PostResponse,
            };

            let input = input.0;
            let created: PgRow;
            let query = {
                if $user_id {
                    crate::post_query!(
                        $model_table;
                        user_id: auth_user.id,
                        $($input_field: input.$input_field),+;
                        "RETURNING id"
                    )
                } else {
                    crate::post_query!(
                        $model_table;
                        $($input_field: input.$input_field),+;
                        "RETURNING id"
                    )
                }
            };
            created = sqlx::query(&query)
                .fetch_one(&mut *db)
                .await
                .map_internal_server_error("Failed to create in database.")?;
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
macro_rules! api_patch {
    (
        model_table: $model_table:expr,
        input: $input:path,
        input_fields: { $($name:ident),+ }
    ) => {
        crate::api_patch! {
            model_table: $model_table,
            input: $input,
            input_fields: { $($name),+ },
            query_where: "WHERE id = $1 AND user_id = $2"
        }
    };
    (
        model_table: $model_table:expr,
        input: $input:path,
        input_fields: { $($name:ident),+ },
        query_where: $query_where:expr
    ) => {
        #[patch("/<id>", data = "<input>", format = "application/json")]
        async fn patch(
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            input: rocket_validation::Validated<rocket::serde::json::Json<$input>>,
            id: uuid::Uuid,
        ) -> crate::responses::APIResult {
            use crate::responses::{bad_request, internal_server_error, result_not_found, ok, result_bad_request};

            let input = input.0;
            let update_str = crate::update_set! {
                $model_table;
                $($name: input.$name),+;
                $query_where
            };
            if let Some(update_str) = update_str {
                let result = sqlx::query(&update_str)
                    .bind(id)
                    .bind(auth_user.id)
                    .execute(&mut *db)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::Database(_) => bad_request("Invalid patch request."),
                        _ => internal_server_error("Failed to patch in database."),
                    })?;
                if result.rows_affected() == 0 {
                    return result_not_found("Item not found.");
                }
            } else {
                return result_bad_request("Empty patch request.");
            }
            Ok(ok("Patch successful."))
        }
    }
}

#[macro_export]
macro_rules! api_delete {
    (
        model_table: $model_table:expr
    ) => {
        crate::api_delete! {
            model_table: $model_table,
            query_where: "WHERE id = $1 AND user_id = $2"
        }
    };
    (
        model_table: $model_table:expr,
        query_where: $query_where:expr
    ) => {
        #[delete("/<id>")]
        async fn delete(
            auth_user: crate::guards::auth::Auth<crate::models::user::UserModel>,
            mut db: rocket_db_pools::Connection<crate::database::BackendDb>,
            id: uuid::Uuid,
        ) -> crate::responses::APIResult {
            use crate::responses::{ok, result_not_found, MapAPIResponse};

            static QUERY_STRING: Lazy<String> =
                Lazy::new(|| format!("DELETE FROM {} {}", $model_table, $query_where));
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
macro_rules! api_crud {
    (
        model_table: $model_table:expr,
        model_fields: { $($model_field:ident),+ },
        get_model: $get_model:path,
        post_input: $post_input:path,
        patch_input: $patch_input:path
    ) => {
        crate::api_crud!(
            model_table: $model_table,
            get: {
                model_type: $get_model,
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
        crate::api_get! {
            model_table: $model_table,
            model_type: $get_model
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
