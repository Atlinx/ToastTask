#![cfg(test)]

// TODO AFTER TESETING: Make a single model integration test into a macro

pub mod extra {
    use reqwest::StatusCode;

    use crate::{
        api::{
            auth::email::utils::email_register_and_login_user, list::utils::setup_lists_default,
        },
        commons,
    };

    #[rocket::async_test]
    async fn get_list_individual_other_unauth() {
        let (client, app) = commons::setup().await;

        let alice_session_response = email_register_and_login_user(&client, "alice").await;

        let bob_session_response = email_register_and_login_user(&client, "bob").await;
        let bob_list_ids = setup_lists_default(&client, &bob_session_response).await;

        let res = client
            .get(&format!("lists/{}", bob_list_ids.first().unwrap()))
            .bearer_auth(alice_session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        app.shutdown().await;
    }
}

pub mod post {
    use super::utils::PostListResponse;
    use crate::{api::auth::email::utils::email_register_and_login_user_default, commons};
    use reqwest::StatusCode;
    use serde_json::json;

    #[rocket::async_test]
    async fn post_list_unauth() {
        let (client, app) = commons::setup().await;
        let session_response = email_register_and_login_user_default(&client).await;
        let res = client
            .post("/lists")
            .json(&json!({
              "title": "Grocery list",
              "color": "sdfsdf",
            }))
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        app.shutdown().await;
    }

    macro_rules! post_list {
      ($($name:ident: $input:expr,)*) => {
      $(
          #[rocket::async_test]
          async fn $name() {
              let (json, status) = $input;
              let (client, app) = commons::setup().await;
              let res = client
                  .post("/lists")
                  .json(&json)
                  .send()
                  .await
                  .expect("Expected response");
              assert_eq!(res.status(), status);
              res.json::<PostListResponse>().await.expect("Expected correct json response");
              app.shutdown().await;
          }
      )*
      }
    }

    post_list! {
      post_list_valid_0: (json!({
        "title": "Grocery list",
        "description": "List of groceries for next week's event.",
        "color": "#ffa783",
      }), StatusCode::CREATED),
      post_list_valid_1: (json!({
        "title": "Grocery list",
        "color": "#ffa783",
      }), StatusCode::CREATED),

      post_list_invalid_0: (json!({
        "title": "Grocery list",
        "color": true,
      }), StatusCode::UNPROCESSABLE_ENTITY),
      post_list_invalid_1: (json!({
        "title": "Grocery list",
        "color": "sdfsdf",
      }), StatusCode::BAD_REQUEST),
    }
}

pub mod get {
    pub mod get_individual {
        use assert_json_diff::assert_json_include;
        use reqwest::StatusCode;
        use serde_json::Value;
        use uuid::Uuid;

        use crate::{
            api::list::utils::{rud_setup, DEFAULT_LISTS},
            commons,
        };

        #[rocket::async_test]
        async fn get_list_individual_unauth() {
            let (client, app) = commons::setup().await;
            let (_, list_ids) = rud_setup(&client).await;
            let res = client
                .get(&format!("lists/{}", list_ids.first().unwrap()))
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            app.shutdown().await;
        }

        #[rocket::async_test]
        async fn get_list_individual() {
            let (client, app) = commons::setup().await;
            let (session_response, list_ids) = rud_setup(&client).await;

            for (id, default_list) in list_ids.iter().zip(DEFAULT_LISTS.iter()) {
                let res = client
                    .get(&format!("lists/{}", id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                let json_res = res.json::<Value>().await.expect("Expected json response");
                assert_json_include!(
                  actual: json_res,
                  expected: default_list
                );
            }

            app.shutdown().await;
        }

        #[rocket::async_test]
        async fn get_list_individual_missing() {
            let (client, app) = commons::setup().await;
            let (session_response, _) = rud_setup(&client).await;
            let res = client
                .get(&format!("lists/{}", Uuid::new_v4()))
                .bearer_auth(session_response.session_token)
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
            app.shutdown().await;
        }
    }

    pub mod get_all {
        use reqwest::StatusCode;
        use uuid::Uuid;

        use crate::{
            api::{
                auth::email::utils::SessionResponse,
                list::utils::{rud_setup, GetListResponse},
                utils::GetAllResponse,
            },
            commons::{self, http_client::HttpClient},
        };

        #[rocket::async_test]
        async fn get_list_all_unauth() {
            let (client, app) = commons::setup().await;
            let _ = rud_setup(&client).await;
            let res = client.get("lists").send().await.expect("Expected response");
            assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
            app.shutdown().await;
        }

        async fn assert_get_list_resp_valid(
            client: &HttpClient,
            session_response: &SessionResponse,
            all_lists: Vec<GetListResponse>,
            list_ids: Vec<Uuid>,
        ) {
            for (list_id, list) in list_ids.iter().zip(all_lists.iter()) {
                let single_fetch_list = client
                    .get(&format!("lists/{}", list_id))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response")
                    .json::<GetListResponse>()
                    .await
                    .expect("Expected json response");
                assert_eq!(single_fetch_list, *list);
            }
        }

        #[rocket::async_test]
        async fn get_list_all() {
            let (client, app) = commons::setup().await;
            let (session_response, list_ids) = rud_setup(&client).await;

            let res = client
                .get("lists")
                .bearer_auth(session_response.session_token)
                .send()
                .await
                .expect("Expected response");
            let get_all = res
                .json::<GetAllResponse<GetListResponse>>()
                .await
                .expect("Expected json response");

            assert_get_list_resp_valid(&client, &session_response, get_all.items, list_ids).await;
            app.shutdown().await;
        }

        #[rocket::async_test]
        async fn get_list_all_paginate() {
            let (client, app) = commons::setup().await;
            let (session_response, list_ids) = rud_setup(&client).await;

            let mut all_lists = Vec::<GetListResponse>::new();
            let mut page_idx = 0;
            loop {
                let res = client
                    .get(&format!("lists/?limit={}&page={}", 3, page_idx))
                    .bearer_auth(session_response.session_token)
                    .send()
                    .await
                    .expect("Expected response");
                let mut get_all = res
                    .json::<GetAllResponse<GetListResponse>>()
                    .await
                    .expect("Expected json response");
                if get_all.items.len() == 0 {
                    break;
                } else {
                    all_lists.append(&mut get_all.items);
                }
                page_idx += 1;
            }

            assert_get_list_resp_valid(&client, &session_response, all_lists, list_ids).await;
            app.shutdown().await;
        }
    }
}

pub mod put {
    use assert_json_diff::assert_json_include;
    use reqwest::StatusCode;
    use serde_json::{json, Value};
    use uuid::Uuid;

    use crate::{api::list::utils::rud_setup, commons};

    #[rocket::async_test]
    async fn put_list_unauth() {
        let (client, app) = commons::setup().await;
        let _ = rud_setup(&client).await;
        let res = client.put("lists").send().await.expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        app.shutdown().await;
    }

    #[rocket::async_test]
    async fn put_list_valid() {
        let (client, app) = commons::setup().await;
        let (session_response, list_ids) = rud_setup(&client).await;
        let list_id = list_ids.first().unwrap();
        let changes = json!({
          "id": list_id,
          "title": "My updated list",
          "description": "This is an updated list",
          "color": "#444488",
        });
        let res = client
            .put("lists")
            .bearer_auth(session_response.session_token)
            .json(&changes)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
        let refetch_item = client
            .get(&format!("lists/{}", list_id))
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
        app.shutdown().await;
    }

    macro_rules! put_list_invalid {
      ($($name:ident: $input:expr,)*) => {
      $(
          #[rocket::async_test]
          async fn $name() {
              let (json, status) = $input;
              let (client, app) = commons::setup().await;
              let (session_response, _) = rud_setup(&client).await;
              let res = client
                  .put("/lists")
                  .json(&json)
                  .bearer_auth(session_response.session_token)
                  .send()
                  .await
                  .expect("Expected response");
              assert_eq!(res.status(), status);
              app.shutdown().await;
          }
      )*
      }
    }

    put_list_invalid! {
      put_list_invalid_0: (json!({
        "title": "My updated list",
        "description": "This is an updated list",
        "color": "#444488",
      }), StatusCode::UNPROCESSABLE_ENTITY),
      put_list_invalid_1: (json!({}), StatusCode::UNPROCESSABLE_ENTITY),
      put_list_invalid_2: (json!({
        "id": "not a uid",
        "description": "This is an updated list",
      }), StatusCode::BAD_REQUEST),
      put_list_missing: (json!({
        "id": Uuid::new_v4(),
      }), StatusCode::NOT_FOUND),
    }
}

pub mod delete {
    use reqwest::StatusCode;
    use uuid::Uuid;

    use crate::{api::list::utils::rud_setup, commons};

    #[rocket::async_test]
    async fn delete_list_unauth() {
        let (client, app) = commons::setup().await;
        let (_, list_ids) = rud_setup(&client).await;
        let list_id = list_ids.first().unwrap();
        let res = client
            .delete(&format!("lists/{}", list_id))
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        app.shutdown().await;
    }

    #[rocket::async_test]
    async fn delete_list_missing() {
        let (client, app) = commons::setup().await;
        let (_, _) = rud_setup(&client).await;
        let invalid_list_id = Uuid::new_v4();
        let res = client
            .delete(&format!("lists/{}", invalid_list_id))
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        app.shutdown().await;
    }

    #[rocket::async_test]
    async fn delete_list() {
        let (client, app) = commons::setup().await;
        let (session_response, list_ids) = rud_setup(&client).await;
        let list_id = list_ids.first().unwrap();
        let confirm_exists_res = client
            .get(&format!("lists/{}", list_id))
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(confirm_exists_res.status(), StatusCode::OK);
        let res = client
            .delete(&format!("lists/{}", list_id))
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
        let confirm_deleted_res = client
            .get(&format!("lists/{}", list_id))
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(confirm_deleted_res.status(), StatusCode::NOT_FOUND);
        app.shutdown().await;
    }
}

pub mod utils {
    use once_cell::sync::Lazy;
    use reqwest::StatusCode;
    use serde::Deserialize;
    use serde_json::{json, Value};
    use uuid::Uuid;

    use crate::{
        api::auth::email::utils::{
            email_register_and_login_user, email_register_and_login_user_default, SessionResponse,
        },
        commons::http_client::HttpClient,
    };

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    pub struct PostListResponse {
        id: Uuid,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    pub struct GetListResponse {
        id: Uuid,
        user: Uuid,
        title: String,
        description: Option<String>,
        color: String,
    }

    pub static DEFAULT_LISTS: Lazy<Vec<Value>> = Lazy::new(|| {
        vec![
            json!({
              "title": "Grocery list",
              "description": "List of groceries for next week's event.",
              "color": "#ffa783",
            }),
            json!({
              "title": "Todo list",
              "description": "Dail for next week's event.",
              "color": "#ba87783",
            }),
            json!({
              "title": "Homework list",
              "color": "#89a783",
            }),
            json!({
              "title": "Birthday list",
              "color": "#370073",
            }),
        ]
    });

    /// Sets up the backend to run tests on read, update, and delete operations
    pub async fn rud_setup(client: &HttpClient) -> (SessionResponse, Vec<Uuid>) {
        // Other user's data, which should be irrelevant
        for i in 0..10 {
            let session_response =
                email_register_and_login_user(client, &format!("alex{}", i)).await;
            setup_lists_default(client, &session_response).await;
        }

        let session_response = email_register_and_login_user_default(client).await;
        let list_ids = setup_lists_default(client, &session_response).await;

        (session_response, list_ids)
    }

    /// Posts a default collection of lists into the API, and returns the ids of the posted lists.
    pub async fn setup_lists_default(
        client: &HttpClient,
        session_response: &SessionResponse,
    ) -> Vec<Uuid> {
        setup_lists(client, session_response, &DEFAULT_LISTS).await
    }

    /// Posts a collection of lists into the API, and returns the ids of the posted lists.
    pub async fn setup_lists(
        client: &HttpClient,
        session_response: &SessionResponse,
        reqs: &Vec<Value>,
    ) -> Vec<Uuid> {
        let mut vec = Vec::<Uuid>::new();
        for req in reqs {
            let res = client
                .post("lists")
                .bearer_auth(session_response.session_token)
                .json(&req)
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), StatusCode::CREATED);
            let response = res
                .json::<PostListResponse>()
                .await
                .expect("Expected correct json response");
            vec.push(response.id);
        }
        vec
    }
}
