// TODO AFTER TESETING: Make a single model integration test into a macro

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
            
                    let alice_session_response = email_register_and_login_user(&client, "alice").await;
            
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
                    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
                }

                async fn assert_get_resp_valid(
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    all_items: Vec<ResponseType>,
                    item_ids: Vec<Uuid>,
                ) {
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
macro_rules! test_patch {
    (
        model_path: $model_path:expr,
        valid_changes: $valid_changes:expr,
        test_cases: { $($test_case_name:ident: $test_case_input:expr,)* },
        rud_setup: $rud_setup:path
    ) => {
        pub mod patch {
            use reqwest::StatusCode;
            use serde_json::json;
            use uuid::Uuid;

            use super::{$rud_setup};
            use crate::commons;

            #[rocket::async_test]
            async fn patch_unauth() {
                let client = commons::setup().await;
                let (_, item_ids, _) = rud_setup(&client).await;
                let item_id = item_ids.first().unwrap();
                let res = client
                    .patch(&format!("{}/{}", $model_path, item_id))
                    .json(&$valid_changes)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            }

            #[rocket::async_test]
            async fn patch_missing() {
                let client = commons::setup().await;
                let (session_response, _, _) = rud_setup(&client).await;
                let res = client
                    .patch(&format!("{}/{}", $model_path, Uuid::new_v4()))
                    .bearer_auth(session_response.session_token)
                    .json(&$valid_changes)
                    .send()
                    .await
                    .expect("Expected response");
                assert_eq!(res.status(), StatusCode::NOT_FOUND);
            }

            mod test_case {
                use reqwest::StatusCode;
                use serde_json::{json, Value};

                use assert_json_diff::assert_json_include;
                use super::super::{$rud_setup};
                use crate::commons;

                $(
                    #[rocket::async_test]
                    async fn $test_case_name() {
                        let (changes, status) = $test_case_input;
                        let client = commons::setup().await;
                        let (session_response, item_ids, _) = rud_setup(&client).await;
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
        rud_setup: $rud_setup:path,
        get: {
            response_type: $get_response_type:path
        },
        post: {
            valid_item: $post_valid_item:expr,
            test_cases: { $($post_test_case_name:ident: $post_test_case_input:expr,)* }
        },
        patch: {
            valid_changes: $patch_valid_changes:expr,
            test_cases: { $($patch_test_case_name:ident: $patch_test_case_input:expr,)* }
        }
    ) => {
        crate::test_post!(
            model_path: $model_path,
            valid_item: $post_valid_item,
            test_cases: {$($post_test_case_name: $post_test_case_input,)* }
        );
        crate::test_get!(
            model_path: $model_path,
            response_type: $get_response_type,
            rud_setup: $rud_setup
        );
        crate::test_patch!(
            model_path: $model_path,
            valid_changes: $patch_valid_changes,
            test_cases: {$($patch_test_case_name: $patch_test_case_input,)*},
            rud_setup: $rud_setup
        );
        crate::test_delete!(
            model_path: $model_path,
            rud_setup: $rud_setup
        );
    };
    (
        model_plural: $model_plural:ident,
        get: {
            response_type: $get_response_type:path
        },
        post: {
            valid_item: $post_valid_item:expr,
            test_cases: { $($post_test_case_name:ident: $post_test_case_input:expr,)* }
        },
        patch: {
            valid_changes: $patch_valid_changes:expr,
            test_cases: { $($patch_test_case_name:ident: $patch_test_case_input:expr,)* }
        },
        default_items: { $($default_item:expr),+ }
    ) => {
        crate::test_crud! {
            model_path: stringify!($model_plural),
            rud_setup: utils::rud_setup,
            get: {
                response_type: $get_response_type
            },
            post: {
                valid_item: $post_valid_item,
                test_cases: { $($post_test_case_name: $post_test_case_input,)* }
            },
            patch: {
                valid_changes: $patch_valid_changes,
                test_cases: { $($patch_test_case_name: $patch_test_case_input,)* }
            }
        }
        crate::test_crud_utils! {
            model_plural: $model_plural,
            default_items: $($default_item),+
        }
    };
}

#[macro_export]
macro_rules! test_crud_utils {
    (
        model_plural: $model_plural:ident,
        default_items: $($default_item:expr),+
    ) => {
        paste::paste!{
            pub mod utils {
                use once_cell::sync::Lazy;
                use reqwest::StatusCode;
                use serde_json::{json, Value};
                use uuid::Uuid;

                use crate::{
                    api::{
                        auth::email::utils::{
                            email_register_and_login_user, email_register_and_login_user_default,
                            SessionResponse,
                        },
                        utils::PostResponse,
                    },
                    commons::http_client::HttpClient,
                };

                pub static [<DEFAULT_ $model_plural:upper>]: Lazy<Vec<Value>> = Lazy::new(|| {
                    vec![$($default_item),+]
                });

                /// Sets up the backend to run tests on read, update, and delete operations
                pub async fn rud_setup(
                    client: &HttpClient,
                ) -> (SessionResponse, Vec<Uuid>, &Vec<serde_json::Value>) {
                    // Other user's data, which should be irrelevant
                    for i in 0..10 {
                        let session_response =
                            email_register_and_login_user(client, &format!("alex{}", i)).await;
                        setup_lists_default(client, &session_response).await;
                    }

                    let session_response = email_register_and_login_user_default(client).await;
                    let list_ids = [<setup_ $model_plural _default>](client, &session_response).await;

                    (session_response, list_ids, &[<DEFAULT_ $model_plural:upper>])
                }

                /// Posts a default collection of lists into the API, and returns the ids of the posted lists.
                pub async fn [<setup_ $model_plural _default>](
                    client: &HttpClient,
                    session_response: &SessionResponse,
                ) -> Vec<Uuid> {
                    [<setup_ $model_plural>](client, session_response, &[<DEFAULT_ $model_plural:upper>]).await
                }

                /// Posts a collection of lists into the API, and returns the ids of the posted lists.
                pub async fn [<setup_ $model_plural>](
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    reqs: &Vec<Value>,
                ) -> Vec<Uuid> {
                    let mut vec = Vec::<Uuid>::new();
                    for req in reqs {
                        let res = client
                            .post("/lists")
                            .bearer_auth(session_response.session_token)
                            .json(&req)
                            .send()
                            .await
                            .expect("Expected response");
                        assert_eq!(res.status(), StatusCode::CREATED);
                        let response = res
                            .json::<PostResponse>()
                            .await
                            .expect("Expected correct json response");
                        vec.push(response.id);
                    }
                    vec
                }
            }
        }
    }
}