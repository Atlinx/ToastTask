/// Generates tests for models that have
/// a get endpoint.
#[macro_export]
macro_rules! test_get {
    (
        model_path: $model_path:expr, 
        response_type: $response_type:path, 
        rud_setup: $rud_setup:path
    ) => {
        pub mod get {
            pub mod single {
                use assert_json_diff::assert_json_include;
                use reqwest::StatusCode;
                use uuid::Uuid;

                use super::super::{$rud_setup, $response_type as ResponseType};
                use crate::{api::auth::email::utils::email_register_and_login_user, commons};

                #[rocket::async_test]
                async fn get_individual_unauth() {
                    let client = commons::setup().await;
                    let (_, item_ids, _) = rud_setup(&client).await;
                    let res = client
                        .get(&format!("{}/{}", $model_path, item_ids.first().unwrap()))
                        .send()
                        .await
                        .expect("Expected response");
                    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
                }

                #[rocket::async_test]
                async fn get_single() {
                    let client = commons::setup().await;
                    let (session_response, item_ids, item_jsons) = rud_setup(&client).await;

                    for (id, default_item) in item_ids.iter().zip(item_jsons.iter()) {
                        let res = client
                            .get(&format!("{}/{}", $model_path, id))
                            .bearer_auth(session_response.session_token)
                            .send()
                            .await
                            .expect("Expected response");
                        let json_res = res.json::<ResponseType>().await.expect("Expected json response");
                        assert_json_include!(
                            actual: serde_json::value::to_value(json_res).unwrap(),
                            expected: default_item
                        );
                    }
                }

                #[rocket::async_test]
                async fn get_single_other_unauth() {
                    let client = commons::setup().await;
            
                    let (alice_session_response, _) = email_register_and_login_user(&client, "alice").await;
            
                    let (_, _, bob_item_ids) = rud_setup(&client).await;
            
                    let res = client
                        .get(&format!("{}/{}", $model_path, bob_item_ids.first().unwrap()))
                        .bearer_auth(alice_session_response.session_token)
                        .send()
                        .await
                        .expect("Expected response");
                    assert_eq!(res.status(), StatusCode::NOT_FOUND);
                }

                #[rocket::async_test]
                async fn get_single_missing() {
                    let client = commons::setup().await;
                    let (session_response, _, _) = rud_setup(&client).await;
                    let res = client
                        .get(&format!("{}/{}", $model_path, Uuid::new_v4()))
                        .bearer_auth(session_response.session_token)
                        .send()
                        .await
                        .expect("Expected response");
                    assert_eq!(res.status(), StatusCode::NOT_FOUND);
                }
            }

            pub mod all {
                use reqwest::StatusCode;
                use uuid::Uuid;

                use super::super::{$rud_setup, $response_type as ResponseType};
                use crate::{
                    api::auth::email::utils::SessionResponse,
                    commons::{self, http_client::HttpClient, utils::rest::GetAllResponse},
                };

                #[rocket::async_test]
                async fn get_all_unauth() {
                    let client = commons::setup().await;
                    let _ = rud_setup(&client).await;
                    let res = client.get($model_path).send().await.expect("Expected response");
                    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
                }

                async fn assert_get_resp_valid(
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    mut all_items: Vec<ResponseType>,
                    mut item_ids: Vec<Uuid>,
                ) {
                    item_ids.sort();
                    all_items.sort_by(|a, b| a.id.cmp(&b.id));

                    assert_eq!(item_ids.len(), all_items.len());

                    for (item_id, item) in item_ids.iter().zip(all_items.iter()) {
                        let single_fetch_item = client
                            .get(&format!("{}/{}", $model_path, item_id))
                            .bearer_auth(session_response.session_token)
                            .send()
                            .await
                            .expect("Expected response")
                            .json::<ResponseType>()
                            .await
                            .expect("Expected json response");
                        assert_eq!(single_fetch_item, *item);
                    }
                }

                #[rocket::async_test]
                async fn get_all() {
                    let client = commons::setup().await;
                    let (session_response, item_ids, _) = rud_setup(&client).await;

                    let res = client
                        .get($model_path)
                        .bearer_auth(session_response.session_token)
                        .send()
                        .await
                        .expect("Expected response");
                    let get_all = res
                        .json::<GetAllResponse<ResponseType>>()
                        .await
                        .expect("Expected json response");
                    
                    assert_get_resp_valid(&client, &session_response, get_all.items, item_ids).await;
                }

                #[rocket::async_test]
                async fn get_all_paginate() {
                    let client = commons::setup().await;
                    let (session_response, item_ids, _) = rud_setup(&client).await;

                    let mut all_items = Vec::<ResponseType>::new();
                    let mut page_idx = 0;
                    loop {
                        let res = client
                            .get(&format!("{}/?limit={}&page={}", $model_path, 3, page_idx))
                            .bearer_auth(session_response.session_token)
                            .send()
                            .await
                            .expect("Expected response");
                        let mut get_all = res
                            .json::<GetAllResponse<ResponseType>>()
                            .await
                            .expect("Expected json response");
                        if get_all.items.len() == 0 {
                            break;
                        } else {
                            all_items.append(&mut get_all.items);
                        }
                        page_idx += 1;
                    }

                    assert_get_resp_valid(&client, &session_response, all_items, item_ids).await;
                }
            }
        }
    }
}

/// Generates tests for models that have
/// a post endpoint.
#[macro_export]
macro_rules! test_post {
    (
        model_path: $model_path:expr,
        valid_item($valid_item_arg_client:ident, $valid_item_arg_session_response:ident) $vaild_item_body:expr,
        test_cases: { $($test_case_name:ident($test_case_arg_client:ident, $test_case_arg_session_response:ident, $test_case_status:expr) $test_case_body:expr),* }
    ) => {
        pub mod post {
            use crate::{api::auth::email::utils::email_register_and_login_user_default, commons};
            use crate::commons::http_client::HttpClient;
            use crate::api::auth::email::utils::SessionResponse;
            use reqwest::StatusCode;
            use serde_json::json;

            #[rocket::async_test]
            async fn post_unauth() {
                let client = commons::setup().await;
                let (session_response, _) = email_register_and_login_user_default(&client).await;
                async fn setup_json($valid_item_arg_client: &HttpClient, $valid_item_arg_session_response: &SessionResponse) -> serde_json::Value {
                    $vaild_item_body
                }
                let valid_item = setup_json(&client, &session_response).await;
                let res = client
                    .post($model_path)
                    .json(&valid_item)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            }

            pub mod test_case {
                use crate::{
                    api::{
                        auth::email::utils::email_register_and_login_user_default
                    }, 
                    commons::{
                        self,
                        utils::rest::PostResponse
                    }
                };
                use reqwest::StatusCode;
                use serde_json::json;
                use crate::commons::http_client::HttpClient;
                use crate::api::auth::email::utils::SessionResponse;

                $(
                    #[rocket::async_test]
                    async fn $test_case_name() {
                        let client = commons::setup().await;
                        let (session_response, _) = email_register_and_login_user_default(&client).await;
                        async fn setup_json($test_case_arg_client: &HttpClient, $test_case_arg_session_response: &SessionResponse) -> serde_json::Value {
                            $test_case_body
                        }
                        let status = $test_case_status;
                        let json = setup_json(&client, &session_response).await;
                        let res = client
                            .post($model_path)
                            .bearer_auth(session_response.session_token)
                            .json(&json)
                            .send()
                            .await
                            .expect("Expected response");
                        assert_eq!(res.status(), status);
                        if status == StatusCode::CREATED {
                            res.json::<PostResponse>().await.expect("Expected correct json response");
                        }
                    }
                )*
            }
        }
    };
}

/// Generates tests for models that have
/// a patch endpoint.
#[macro_export]
macro_rules! test_patch {
    (
        model_path: $model_path:expr,
        valid_changes($valid_changes_arg_client:ident, $valid_changes_arg_session_response:ident) $valid_changes_body:expr,
        test_cases: { $($test_case_name:ident($test_case_arg_client:ident, $test_case_arg_session_response:ident, $test_case_status:expr) $test_case_body:expr),* }
        rud_setup: $rud_setup:path
    ) => {
        pub mod patch {
            use reqwest::StatusCode;
            use serde_json::json;
            use uuid::Uuid;
            
            use super::{$rud_setup as rud_setup};
            use crate::commons;
            use crate::commons::http_client::HttpClient;
            use crate::api::auth::email::utils::SessionResponse;

            #[rocket::async_test]
            async fn patch_unauth() {
                let client = commons::setup().await;
                let (session_response, item_ids, _) = rud_setup(&client).await;
                async fn setup_json($valid_changes_arg_client: &HttpClient, $valid_changes_arg_session_response: &SessionResponse) -> serde_json::Value {
                    $valid_changes_body
                }
                let valid_changes = setup_json(&client, &session_response).await;
                let item_id = item_ids.first().unwrap();                
                let res = client
                    .patch(&format!("{}/{}", $model_path, item_id))
                    .json(&valid_changes)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            }

            #[rocket::async_test]
            async fn patch_missing() {
                let client = commons::setup().await;
                let (session_response, _, _) = rud_setup(&client).await;
                async fn setup_json($valid_changes_arg_client: &HttpClient, $valid_changes_arg_session_response: &SessionResponse) -> serde_json::Value {
                    $valid_changes_body
                }
                let valid_changes = setup_json(&client, &session_response).await;
                let res = client
                    .patch(&format!("{}/{}", $model_path, Uuid::new_v4()))
                    .bearer_auth(session_response.session_token)
                    .json(&valid_changes)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::NOT_FOUND);
            }

            mod test_case {
                use reqwest::StatusCode;
                use serde_json::{json, Value};

                use assert_json_diff::assert_json_include;
                use super::super::{$rud_setup as rud_setup};
                use crate::commons;
                use crate::commons::http_client::HttpClient;
                use crate::api::auth::email::utils::SessionResponse;

                $(
                    #[rocket::async_test]
                    async fn $test_case_name() {
                        let client = commons::setup().await;
                        let (session_response, item_ids, _) = rud_setup(&client).await;
                        let item_id = item_ids.first().unwrap();
                        async fn setup_json($test_case_arg_client: &HttpClient, $test_case_arg_session_response: &SessionResponse) -> serde_json::Value {
                            $test_case_body
                        }
                        let changes = setup_json(&client, &session_response).await;
                        let status = $test_case_status;
                        let res = client
                            .patch(&format!("{}/{}", $model_path, item_id))
                            .json(&changes)
                            .bearer_auth(session_response.session_token)
                            .send()
                            .await
                            .expect("Expected response");
                        assert_eq!(res.status(), status);
                        if status == StatusCode::OK {
                            let refetch_item = client
                                .get(&format!("{}/{}", $model_path, item_id))
                                .bearer_auth(session_response.session_token)
                                .send()
                                .await
                                .expect("Expected response")
                                .json::<Value>()
                                .await
                                .expect("Expected json response");
                            assert_json_include!(
                                actual: refetch_item,
                                expected: changes
                            );
                        }
                    }
                )*
            }
        }
    }
}

/// Generates tests for models that have
/// a delete endpoint.
#[macro_export]
macro_rules! test_delete {
    (
        model_path: $model_path:expr, 
        rud_setup: $rud_setup:path
    ) => {
        pub mod delete {
            use super::{$rud_setup};
            use reqwest::StatusCode;
            use uuid::Uuid;

            use crate::commons;

            #[rocket::async_test]
            async fn delete_unauth() {
                let client = commons::setup().await;
                let (_, item_ids, _) = rud_setup(&client).await;
                let item_id = item_ids.first().unwrap();
                let res = client
                    .delete(&format!("{}/{}", $model_path, item_id))
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            }

            #[rocket::async_test]
            async fn delete_missing() {
                let client = commons::setup().await;
                let (session_response, _, _) = rud_setup(&client).await;
                let invalid_item_id = Uuid::new_v4();
                let res = client
                    .delete(&format!("{}/{}", $model_path, invalid_item_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::NOT_FOUND);
            }

            #[rocket::async_test]
            async fn delete() {
                let client = commons::setup().await;
                let (session_response, item_ids, _) = rud_setup(&client).await;
                let item_id = item_ids.first().unwrap();
                println!("delete 1");
                let confirm_exists_res = client
                    .get(&format!("{}/{}", $model_path, item_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                let status = confirm_exists_res.status();
                assert_eq!(status, StatusCode::OK);
                println!("delete 2");
                let res = client
                    .delete(&format!("{}/{}", $model_path, item_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::OK);
                println!("delete 3");
                let confirm_deleted_res = client
                    .get(&format!("{}/{}", $model_path, item_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(confirm_deleted_res.status(), StatusCode::NOT_FOUND);
                println!("delete 4");
            }
        }
    };
}

/// Generates tests for models that have
/// a CRUD endpoint.
#[macro_export]
macro_rules! test_crud {
    (
        model_path: $model_path:expr,
        rud_setup: $rud_setup:path,
        get: {
            response_type: $get_response_type:path
        },
        post: {
            valid_item($post_valid_item_arg_client:ident, $post_valid_item_arg_session_response:ident) $post_vaild_item_body:expr,
            test_cases: { $($post_test_case_name:ident($post_test_case_arg_client:ident, $post_test_case_arg_session_response:ident, $post_test_case_status:expr) $post_test_case_body:expr),* }
        },
        patch: {
            valid_changes($patch_valid_changes_arg_client:ident, $patch_valid_changes_arg_session_response:ident) $patch_valid_changes_body:expr,
            test_cases: { $($patch_test_case_name:ident($patch_test_case_arg_client:ident, $patch_test_case_arg_session_response:ident, $patch_test_case_status:expr) $patch_test_case_body:expr),* }
        }
    ) => {
        crate::test_get!(
            model_path: $model_path,
            response_type: $get_response_type,
            rud_setup: $rud_setup
        );
        crate::test_post!(
            model_path: $model_path,
            valid_item($post_valid_item_arg_client, $post_valid_item_arg_session_response) $post_vaild_item_body,
            test_cases: { $($post_test_case_name($post_test_case_arg_client, $post_test_case_arg_session_response, $post_test_case_status) $post_test_case_body),* }
        );
        crate::test_patch!(
            model_path: $model_path,
            valid_changes($patch_valid_changes_arg_client, $patch_valid_changes_arg_session_response) $patch_valid_changes_body,
            test_cases: { $($patch_test_case_name($patch_test_case_arg_client, $patch_test_case_arg_session_response, $patch_test_case_status) $patch_test_case_body),* }
            rud_setup: $rud_setup
        );
        crate::test_delete!(
            model_path: $model_path,
            rud_setup: $rud_setup
        );
    };
    (
        model_path: $model_path:expr,
        model_plural: $model_plural:ident,
        get: {
            response_type: $get_response_type:path
        },
        post: {
            valid_item($post_valid_item_arg_client:ident, $post_valid_item_arg_session_response:ident) $post_vaild_item_body:expr,
            test_cases: { $($post_test_case_name:ident($post_test_case_arg_client:ident, $post_test_case_arg_session_response:ident, $post_test_case_status:expr) $post_test_case_body:expr),* }
        },
        patch: {
            valid_changes($patch_valid_changes_arg_client:ident, $patch_valid_changes_arg_session_response:ident) $patch_valid_changes_body:expr,
            test_cases: { $($patch_test_case_name:ident($patch_test_case_arg_client:ident, $patch_test_case_arg_session_response:ident, $patch_test_case_status:expr) $patch_test_case_body:expr),* }
        },
        default_items: { $($default_item:expr),+ }
    ) => {
        crate::test_crud! {
            model_path: $model_path,
            rud_setup: utils::rud_setup,
            get: {
                response_type: $get_response_type
            },
            post: {
                valid_item($post_valid_item_arg_client, $post_valid_item_arg_session_response) $post_vaild_item_body,
                test_cases: { $($post_test_case_name($post_test_case_arg_client, $post_test_case_arg_session_response, $post_test_case_status) $post_test_case_body),* }
            },
            patch: {
                valid_changes($patch_valid_changes_arg_client, $patch_valid_changes_arg_session_response) $patch_valid_changes_body,
                test_cases: { $($patch_test_case_name($patch_test_case_arg_client, $patch_test_case_arg_session_response, $patch_test_case_status) $patch_test_case_body),* }
            }
        }
        crate::test_crud_utils! {
            model_path: $model_path,
            model_plural: $model_plural,
            default_items: { $($default_item),+ }
        }
    };
    (
        model_path: $model_path:expr,
        model_plural: $model_plural:ident,
        get: {
            response_type: $get_response_type:path
        },
        post: {
            valid_item($post_valid_item_arg_client:ident, $post_valid_item_arg_session_response:ident) $post_vaild_item_body:expr,
            test_cases: { $($post_test_case_name:ident($post_test_case_arg_client:ident, $post_test_case_arg_session_response:ident, $post_test_case_status:expr) $post_test_case_body:expr),* }
        },
        patch: {
            valid_changes($patch_valid_changes_arg_client:ident, $patch_valid_changes_arg_session_response:ident) $patch_valid_changes_body:expr,
            test_cases: { $($patch_test_case_name:ident($patch_test_case_arg_client:ident, $patch_test_case_arg_session_response:ident, $patch_test_case_status:expr) $patch_test_case_body:expr),* }
        },
        default_items: { $($default_item:expr),+ },
        rud_setup_items($client:ident, $session_response:ident, $templates:ident ) $rud_setup_items:expr
    ) => {
        crate::test_crud! {
            model_path: $model_path,
            rud_setup: utils::rud_setup,
            get: {
                response_type: $get_response_type
            },
            post: {
                valid_item ($post_valid_item_arg_client, $post_valid_item_arg_session_response) $post_vaild_item_body,
                test_cases: { $($post_test_case_name($post_test_case_arg_client, $post_test_case_arg_session_response, $post_test_case_status) $post_test_case_body),* }
            },
            patch: {
                valid_changes($patch_valid_changes_arg_client, $patch_valid_changes_arg_session_response) $patch_valid_changes_body,
                test_cases: { $($patch_test_case_name($patch_test_case_arg_client, $patch_test_case_arg_session_response, $patch_test_case_status) $patch_test_case_body),* }
            }
        }
        crate::test_crud_utils! {
            model_plural: $model_plural,
            default_items: { $($default_item),+ },
            rud_setup_items($client, $session_response, $templates) $rud_setup_items
        }
    };
}

#[macro_export]
macro_rules! test_crud_utils {
    (
        model_path: $model_path:expr,
        model_plural: $model_plural:ident,
        default_items: { $($default_item:expr),+ }
    ) => {
        crate::test_crud_utils! {
            model_plural: $model_plural,
            default_items: { $($default_item),+ },
            rud_setup_items(client, session_response, templates) {
                use crate::commons::utils::rest::PostResponse;
                use reqwest::StatusCode;
                
                let mut uuid_vec = Vec::<Uuid>::new();
                let mut value_vec = Vec::<Value>::new();
                for template in templates {
                    let req = template.clone();
                    let res = client
                        .post($model_path)
                        .bearer_auth(session_response.session_token)
                        .json(&template)
                        .send()
                        .await
                        .expect("Expected response");
                    assert_eq!(res.status(), StatusCode::CREATED);
                    let response = res
                        .json::<PostResponse>()
                        .await
                        .expect("Expected correct json response");
                    uuid_vec.push(response.id);
                    value_vec.push(req);
                }
                (uuid_vec, value_vec)
            }
        }
    };
    (
        model_plural: $model_plural:ident,
        default_items: { $($default_item:expr),+ },
        rud_setup_items($client:ident, $session_response:ident, $templates:ident ) $rud_setup_items:expr
    ) => {
        paste::paste!{
            pub mod utils {
                use once_cell::sync::Lazy;
                use serde_json::{json, Value};
                use uuid::Uuid;

                use crate::{
                    api::{
                        auth::email::utils::{
                            email_register_and_login_user, email_register_and_login_user_default,
                            SessionResponse,
                        },
                    },
                    commons::http_client::HttpClient,
                };

                pub static [<DEFAULT_ $model_plural:upper _TEMPLATES>]: Lazy<Vec<Value>> = Lazy::new(|| {
                    vec![$($default_item),+]
                });

                /// Sets up the backend to run tests on read, update, and delete operations
                pub async fn rud_setup(
                    client: &HttpClient,
                ) -> (SessionResponse, Vec<Uuid>, Vec<serde_json::Value>) {
                    // Other user's data, which should be irrelevant
                    for i in 0..10 {
                        let (session_response, _) =
                            email_register_and_login_user(client, &format!("alex{}", i)).await;
                        [<setup_ $model_plural _default>](client, &session_response).await;
                    }

                    let (session_response, _) = email_register_and_login_user_default(client).await;
                    let (item_ids, items) = [<setup_ $model_plural _default>](client, &session_response).await;

                    (session_response, item_ids, items)
                }

                pub async fn [<setup_ $model_plural _default>](
                    client: &HttpClient,
                    session_response: &SessionResponse,
                ) -> (Vec<Uuid>, Vec<Value>) {
                    [<setup_ $model_plural>](client, session_response, &[<DEFAULT_ $model_plural:upper _TEMPLATES>]).await
                }

                pub async fn [<setup_ $model_plural>](
                    $client: &HttpClient, 
                    $session_response: &SessionResponse, 
                    $templates: &Vec<Value>
                ) -> (Vec<Uuid>, Vec<Value>) {
                    $rud_setup_items
                }
            }
        }
    }
}