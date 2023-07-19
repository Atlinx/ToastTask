#![cfg(test)]

// TODO AFTER TESETING: Make a single model integration test into a macro

use crate::test_crud;

test_crud! {
    model_path: "lists",
    post: {
        valid_item: json!({
            "title": "Grocery list",
            "color": "sdfsdf",
        }),
        test_cases: {
            valid_0: (json!({
                "title": "Grocery list",
                "description": "List of groceries for next week's event.",
                "color": "#ffa783",
            }), StatusCode::CREATED),
            valid_1: (json!({
                "title": "Grocery list",
                "color": "#ffa783",
            }), StatusCode::CREATED),
            invalid_0: (json!({
                "title": "Grocery list",
                "color": true,
            }), StatusCode::UNPROCESSABLE_ENTITY),
            invalid_1: (json!({
                "title": "Grocery list",
                "color": "sdfsdf",
            }), StatusCode::BAD_REQUEST),
        }
    },
    get: {
        response_type: GetListResponse
    },
    put: {
        changes: json!({
            "title": "My updated list",
            "description": "This is an updated list",
            "color": "#444488",
        }),
        test_cases: {
            invalid_0: (json!({
            "title": "My updated list",
            "description": "This is an updated list",
            "color": "#444488",
            }), StatusCode::UNPROCESSABLE_ENTITY),
            invalid_1: (json!({}), StatusCode::UNPROCESSABLE_ENTITY),
            invalid_2: (json!({
            "id": "not a uid",
            "description": "This is an updated list",
            }), StatusCode::BAD_REQUEST),
            missing: (json!({
            "id": Uuid::new_v4(),
            }), StatusCode::NOT_FOUND),
        }
    },
    delete: {}
}

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
        let client = commons::setup().await;

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
    }
}

pub mod utils {
    use once_cell::sync::Lazy;
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};
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

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct GetListResponse {
        id: Uuid,
        user: Uuid,
        title: String,
        description: Option<String>,
        color: String,
    }

    pub static DEFAULT_ITEMS: Lazy<Vec<Value>> = Lazy::new(|| {
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
        setup_lists(client, session_response, &DEFAULT_ITEMS).await
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
                .json::<PostResponse>()
                .await
                .expect("Expected correct json response");
            vec.push(response.id);
        }
        vec
    }
}
