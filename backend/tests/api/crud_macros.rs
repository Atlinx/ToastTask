// TODO AFTER TESETING: Make a single model integration test into a macro

#[macro_export]
macro_rules! test_post {
    (
        model_path: $model_path:expr,
        valid_item: $valid_item:expr,
        test_cases: { $($test_case_name:ident: $test_case_input:expr,)* }
    ) => {
        pub mod post {
            use crate::{api::auth::email::utils::email_register_and_login_user_default, commons};
            use reqwest::StatusCode;
            use serde_json::json;

            #[rocket::async_test]
            async fn post_unauth() {
                let client = commons::setup().await;
                let _ = email_register_and_login_user_default(&client).await;
                let res = client
                    .post($model_path)
                    .json(&$valid_item)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            }

            pub mod test_case {
                use crate::{api::{utils::PostResponse, auth::email::utils::email_register_and_login_user_default}, commons::self};
                use reqwest::StatusCode;
                use serde_json::json;

                $(
                    #[rocket::async_test]
                    async fn $test_case_name() {
                        let (json, status) = $test_case_input;
                        let client = commons::setup().await;
                        let session_response = email_register_and_login_user_default(&client).await;
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

#[macro_export]
macro_rules! test_get {
    (model_path: $model_path:expr, response_type: $response_type:ident) => {
        pub mod get {
            pub mod single {
                use assert_json_diff::assert_json_include;
                use reqwest::StatusCode;
                use uuid::Uuid;

                use super::super::utils::{rud_setup, $response_type, DEFAULT_ITEMS};
                use crate::commons;

                #[rocket::async_test]
                async fn get_individual_unauth() {
                    let client = commons::setup().await;
                    let (_, item_ids) = rud_setup(&client).await;
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
                    let (session_response, item_ids) = rud_setup(&client).await;

                    for (id, default_item) in item_ids.iter().zip(DEFAULT_ITEMS.iter()) {
                        let res = client
                            .get(&format!("{}/{}", $model_path, id))
                            .bearer_auth(session_response.session_token)
                            .send()
                            .await
                            .expect("Expected response");
                        let json_res = res.json::<$response_type>().await.expect("Expected json response");
                        assert_json_include!(
                            actual: serde_json::value::to_value(json_res).unwrap(),
                            expected: default_item
                        );
                    }
                }

                #[rocket::async_test]
                async fn get_single_missing() {
                    let client = commons::setup().await;
                    let (session_response, _) = rud_setup(&client).await;
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

                use super::super::utils::{rud_setup, $response_type};
                use crate::{
                    api::{
                        auth::email::utils::SessionResponse,
                        utils::GetAllResponse,
                    },
                    commons::{self, http_client::HttpClient},
                };

                #[rocket::async_test]
                async fn get_all_unauth() {
                    let client = commons::setup().await;
                    let _ = rud_setup(&client).await;
                    let res = client.get($model_path).send().await.expect("Expected response");
                    let status = res.status();
                    println!("Resp: {:#?}", res.json::<serde_json::Value>().await.unwrap());
                    assert_eq!(status, StatusCode::UNAUTHORIZED);
                }

                async fn assert_get_resp_valid(
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    all_items: Vec<$response_type>,
                    item_ids: Vec<Uuid>,
                ) {
                    for (item_id, item) in item_ids.iter().zip(all_items.iter()) {
                        let single_fetch_item = client
                            .get(&format!("{}/{}", $model_path, item_id))
                            .bearer_auth(session_response.session_token)
                            .send()
                            .await
                            .expect("Expected response")
                            .json::<$response_type>()
                            .await
                            .expect("Expected json response");
                        assert_eq!(single_fetch_item, *item);
                    }
                }

                #[rocket::async_test]
                async fn get_all() {
                    let client = commons::setup().await;
                    let (session_response, item_ids) = rud_setup(&client).await;

                    let res = client
                        .get($model_path)
                        .bearer_auth(session_response.session_token)
                        .send()
                        .await
                        .expect("Expected response");
                    let get_all = res
                        .json::<GetAllResponse<$response_type>>()
                        .await
                        .expect("Expected json response");

                    assert_get_resp_valid(&client, &session_response, get_all.items, item_ids).await;
                }

                #[rocket::async_test]
                async fn get_all_paginate() {
                    let client = commons::setup().await;
                    let (session_response, item_ids) = rud_setup(&client).await;

                    let mut all_items = Vec::<$response_type>::new();
                    let mut page_idx = 0;
                    loop {
                        let res = client
                            .get(&format!("{}/?limit={}&page={}", $model_path, 3, page_idx))
                            .bearer_auth(session_response.session_token)
                            .send()
                            .await
                            .expect("Expected response");
                        let mut get_all = res
                            .json::<GetAllResponse<$response_type>>()
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

#[macro_export]
macro_rules! test_patch {
    (
        model_path: $model_path:expr,
        valid_changes: $valid_changes:expr,
        test_cases: { $($test_case_name:ident: $test_case_input:expr,)* }
    ) => {
        pub mod patch {
            use reqwest::StatusCode;
            use serde_json::{json, Value};
            use uuid::Uuid;

            use super::utils::rud_setup;
            use crate::commons;

            #[rocket::async_test]
            async fn patch_unauth() {
                let client = commons::setup().await;
                let (_, item_ids) = rud_setup(&client).await;
                let item_id = item_ids.first().unwrap();
                let res = client.patch(&format!("{}/{}", $model_path,item_id)).send().await.expect("Expected response");
                assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            }

            #[rocket::async_test]
            async fn patch_missing() {
                let client = commons::setup().await;
                let (session_response, _) = rud_setup(&client).await;
                let res = client
                    .patch(&format!("{}/{}", $model_path, Uuid::new_v4()))
                    .bearer_auth(session_response.session_token)
                    .json(&$valid_changes)
                    .send()
                    .await
                    .expect("Expected response");
                let status = res.status();
                println!("got response: {:#?}", res.json::<Value>().await.unwrap());
                assert_eq!(status, StatusCode::NOT_FOUND);
            }

            mod test_case {
                use reqwest::StatusCode;
                use serde_json::{json, Value};

                use assert_json_diff::assert_json_include;
                use super::super::utils::rud_setup;
                use crate::commons;

                $(
                    #[rocket::async_test]
                    async fn $test_case_name() {
                        let (changes, status) = $test_case_input;
                        let client = commons::setup().await;
                        let (session_response, item_ids) = rud_setup(&client).await;
                        let item_id = item_ids.first().unwrap();
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

#[macro_export]
macro_rules! test_delete {
    (model_path: $model_path:expr) => {
        pub mod delete {
            use super::utils::rud_setup;
            use reqwest::StatusCode;
            use uuid::Uuid;

            use crate::commons;

            #[rocket::async_test]
            async fn delete_unauth() {
                let client = commons::setup().await;
                let (session_response, item_ids) = rud_setup(&client).await;
                let item_id = item_ids.first().unwrap();
                let res = client
                    .delete(&format!("{}/{}", $model_path, item_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            }

            #[rocket::async_test]
            async fn delete_missing() {
                let client = commons::setup().await;
                let (session_response, _) = rud_setup(&client).await;
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
                let (session_response, item_ids) = rud_setup(&client).await;
                let item_id = item_ids.first().unwrap();
                let confirm_exists_res = client
                    .get(&format!("{}/{}", $model_path, item_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(confirm_exists_res.status(), StatusCode::OK);
                let res = client
                    .delete(&format!("{}/{}", $model_path, item_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::OK);
                let confirm_deleted_res = client
                    .get(&format!("{}/{}", $model_path, item_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(confirm_deleted_res.status(), StatusCode::NOT_FOUND);
            }
        }
    };
}

#[macro_export]
macro_rules! test_crud {
    (
        model_path: $model_path:expr,
        post: {
            valid_item: $post_valid_item:expr,
            test_cases: { $($post_test_case_name:ident: $post_test_case_input:expr,)* }
        },
        get: {
            response_type: $get_response_type:ident
        },
        patch: {
            valid_changes: $patch_valid_changes:expr,
            test_cases: { $($patch_test_case_name:ident: $patch_test_case_input:expr,)* }
        },
        delete: {}
    ) => {
        crate::test_post!(
            model_path: $model_path,
            valid_item: $post_valid_item,
            test_cases: {$($post_test_case_name: $post_test_case_input,)* }
        );
        crate::test_get!(
            model_path: $model_path,
            response_type: $get_response_type
        );
        crate::test_patch!(
            model_path: $model_path,
            valid_changes: $patch_valid_changes,
            test_cases: {$($patch_test_case_name: $patch_test_case_input,)*}
        );
        crate::test_delete!(model_path: $model_path);
    };
}
