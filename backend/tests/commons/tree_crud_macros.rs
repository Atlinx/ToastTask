/// Generates tests for CRUD models
/// that follow a parent/child relationship.
///
/// ## Requirements
///
/// - There must be >= 7 default items returned by rud_setup
/// - The `response_type` struct must have the following fields
///     - children: Vec<Uuid>
///     - parent: Option<Uuid>
#[macro_export]
macro_rules! test_tree_crud {
    (
        model_path: $model_path:expr,
        response_type: $response_type:path,
        valid_item($valid_item_arg_client:ident, $valid_item_arg_session_response:ident) $vaild_item_body:expr,
        rud_setup: $rud_setup:path
    ) => {
        pub mod tree {
            use reqwest::StatusCode;
            use serde_json::json;
            use uuid::Uuid;

            use super::{$rud_setup as rud_setup};
            use crate::commons::{self, utils::rest::PostResponse};
            use crate::commons::http_client::HttpClient;
            use crate::api::auth::email::utils::SessionResponse;

            #[rocket::async_test]
            pub async fn post_with_parent() {
                let client = commons::setup().await;
                let (session_response, item_ids, _) = rud_setup(&client).await;

                utils::assert_detached(&client, &session_response, item_ids[0]).await;

                async fn setup_json($valid_item_arg_client: &HttpClient, $valid_item_arg_session_response: &SessionResponse) -> serde_json::Value {
                    $vaild_item_body
                }

                let mut post_input = setup_json(&client, &session_response).await;
                post_input["parent_id"] = serde_json::to_value(item_ids[0]).unwrap();
                println!("Post parent with {:#?} to path {}", post_input, $model_path);
                let res = client
                    .post($model_path)
                    .json(&post_input)
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected a response");
                assert_eq!(res.status(), StatusCode::CREATED);
                let post_res = res.json::<PostResponse>().await.expect("Expected a json response");

                utils::assert_has_parent(&client, &session_response, post_res.id, Some(item_ids[0])).await;
            }

            #[rocket::async_test]
            pub async fn reparent_invalid() {
                let client = commons::setup().await;
                let (session_response, item_ids, _) = rud_setup(&client).await;
                let first_item_id = item_ids[0];

                // Create the parent-child relationship
                let res = client
                    .patch(&format!("{}/{}", $model_path, first_item_id))
                    .json(&json!({
                        "parent_id": Uuid::new_v4()
                    }))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected a response");
                assert_eq!(res.status(), StatusCode::BAD_REQUEST);
            }

            #[rocket::async_test]
            pub async fn reparent() {
                let client = commons::setup().await;
                let (session_response, item_ids, _) = rud_setup(&client).await;

                // Relationship intiially does not exist
                utils::assert_detached(&client, &session_response, item_ids[0]).await;
                utils::assert_detached(&client, &session_response, item_ids[1]).await;

                // Create the parent-child relationship
                utils::reparent(&client, &session_response, item_ids[0], item_ids[1]).await;

                // Parent-child relationship now exists
                utils::assert_has_parent(&client, &session_response, item_ids[0], Some(item_ids[1])).await;
            }

            #[rocket::async_test]
            pub async fn reparent_complex() {
                let client = commons::setup().await;
                let (session_response, item_ids, _) = rud_setup(&client).await;

                // Relationship intiially does not exist
                utils::assert_detached(&client, &session_response, item_ids[0]).await;
                utils::assert_detached(&client, &session_response, item_ids[1]).await;
                utils::assert_detached(&client, &session_response, item_ids[2]).await;
                utils::assert_detached(&client, &session_response, item_ids[3]).await;
                utils::assert_detached(&client, &session_response, item_ids[4]).await;
                utils::assert_detached(&client, &session_response, item_ids[5]).await;

                // Create the following relationship:
                //
                //     1
                //    / \
                //   0   2
                //      /|\
                //     3 4 5
                //

                utils::reparent(&client, &session_response, item_ids[3], item_ids[2]).await;
                utils::reparent(&client, &session_response, item_ids[4], item_ids[2]).await;
                utils::reparent(&client, &session_response, item_ids[0], item_ids[1]).await;
                utils::reparent(&client, &session_response, item_ids[2], item_ids[1]).await;
                utils::reparent(&client, &session_response, item_ids[5], item_ids[2]).await;

                // Parent-child relationship now exists
                utils::assert_has_parent(&client, &session_response, item_ids[0], Some(item_ids[1])).await;
                utils::assert_has_parent(&client, &session_response, item_ids[2], Some(item_ids[1])).await;
                utils::assert_has_parent(&client, &session_response, item_ids[3], Some(item_ids[2])).await;
                utils::assert_has_parent(&client, &session_response, item_ids[4], Some(item_ids[2])).await;
                utils::assert_has_parent(&client, &session_response, item_ids[5], Some(item_ids[2])).await;
            }

            pub mod utils {
                use reqwest::StatusCode;
                use serde_json::json;
                use uuid::Uuid;

                use super::super::{$response_type as ResponseType};

                use crate::{
                    api::auth::email::utils::SessionResponse,
                    commons::http_client::HttpClient,
                };

                /// Reparents a child item to a new parent.
                pub async fn reparent(
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    child_id: Uuid,
                    parent_id: Uuid
                ) {
                    let res = client
                        .patch(&format!("{}/{}", $model_path, child_id))
                        .json(&json!({
                            "parent_id": parent_id
                        }))
                        .bearer_auth(session_response.session_token)
                        .send()
                        .await
                        .expect("Expected a response");
                    assert_eq!(res.status(), StatusCode::OK);
                }

                /// Fetchs an item.
                pub async fn get_item(
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    item_id: Uuid,
                ) -> ResponseType {
                    let res = client
                        .get(&format!("{}/{}", $model_path, item_id))
                        .bearer_auth(session_response.session_token)
                        .send()
                        .await
                        .expect("Expected response")
                        .json::<ResponseType>()
                        .await
                        .expect("Expected json response");
                    res
                }

                /// Asserts that a child item has a parent.
                pub async fn assert_has_parent(
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    child_id: Uuid,
                    parent_id: Option<Uuid>
                ) {
                    if let Some(parent_id) = parent_id {
                        assert_ne!(child_id, parent_id, "Cannot assert parent_id = child_id");
                    }
                    let child = get_item(&client, &session_response, child_id).await;
                    assert_eq!(
                        child.parent_id,
                        parent_id,
                        "Expected child to have parent"
                    );
                    if let Some(parent_id) = parent_id {
                        let parent = get_item(&client, &session_response, parent_id).await;
                        assert!(
                            parent.child_ids.contains(&child_id),
                            "Expected parent to have child \"{}\"",
                            child_id
                        );
                    }
                }

                /// Asserts that an item has no children.
                pub async fn assert_no_children(
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    parent_id: Uuid
                ) {
                    let parent = get_item(&client, &session_response, parent_id).await;
                    assert!(
                        parent.child_ids.is_empty(),
                        "Expected parent to have no children"
                    );
                }

                /// Asserts that an item doesn't have
                /// a parent or children.
                pub async fn assert_detached(
                    client: &HttpClient,
                    session_response: &SessionResponse,
                    item_id: Uuid
                ) {
                    let item = get_item(&client, &session_response, item_id).await;
                    assert!(
                        item.child_ids.is_empty(),
                        "Expected item to have no children"
                    );
                    assert_eq!(
                        item.parent_id,
                        None,
                        "Expected item to have no parent"
                    );
                }
            }
        }
    }
}
