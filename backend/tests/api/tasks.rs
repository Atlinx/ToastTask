#![cfg(test)]

crate::test_crud! {
    model_path: "tasks",
    model_plural: tasks,
    get: {
        response_type: types::GetTaskResponse
    },
    post: {
        valid_item: json!({
            "title": "Grocery list",
            "color": "#ffa783",
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
    patch: {
        valid_changes: json!({
            "title": "My updated list",
            "description": "This is an updated list",
            "color": "#444488",
        }),
        test_cases: {
            valid_0: (json!({
                "title": "My updated list",
                "description": "This is an updated list",
                "color": "#444488",
            }), StatusCode::OK),
            valid_1: (json!({
                "title": "My updated list",
                "description": null
            }), StatusCode::OK),

            invalid_0: (json!({
                "color": null,
            }), StatusCode::BAD_REQUEST),
            invalid_1: (json!({
                "title": null,
                "description": "This is an updated list",
                "color": "#3849dfa"
            }), StatusCode::BAD_REQUEST),
        }
    },
    default_items: {
        json!({
            "due_at": "2023-10-19 10:23:00",
            "due_text": "Next Monday",
            "title": "Go shopping with friends"
        }),
        json!({
            "due_at": "2023-10-29 13:10:00",
            "due_text": "Every Thursday",
            "completed": true,
            "title": "Taking out the garbage"
        }),
        json!({
            "due_at": "2023-10-19 10:00:00",
            "due_text": "Due thursday",
            "completed": true,
            "title": "Attend meeting"
        }),
        json!({
            "due_at": "2023-10-30 15:30:00",
            "due_text": "Due Friday",
            "completed": false,
            "title": "Attend office hours"
        }),
        json!({
            "due_at": "2023-11-3 8:00:00",
            "due_text": "Every Thursday",
            "title": "Call parents"
        }),
        json!({
            "due_at": "2023-11-5 10:23:00",
            "due_text": "November 11",
            "completed": false,
            "title": "Meet with freinds"
        }),
        json!({
            "due_at": "2023-11-6 20:30:00",
            "due_text": "November 11",
            "title": "Prepare birthday gift"
        })
    },
    setup_items_fn: custom_utils::setup_tasks
}

crate::test_parent_child! {
    model_path: "tasks",
    response_type: types::GetTaskResponse,
    valid_item: json!({
        "title": "Grocery list",
        "color": "#ffa783",
    }),
    rud_setup: utils::rud_setup
}

pub mod types {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct GetTaskResponse {
        pub id: Uuid,
        pub list_id: Uuid,
        pub created_at: NaiveDateTime,
        pub updated_at: NaiveDateTime,
        pub due_at: NaiveDateTime,
        pub due_text: String,
        pub completed: bool,
        pub title: String,
        pub description: Option<String>,
        pub parent: Option<Uuid>,
        pub children: Vec<Uuid>,
    }
}

pub mod custom_utils {
    use reqwest::StatusCode;
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{
        api::{auth::email::utils::SessionResponse, lists::utils::setup_lists_default},
        commons::{http_client::HttpClient, utils::rest::PostResponse},
    };

    pub async fn setup_tasks(
        client: &HttpClient,
        session_response: &SessionResponse,
        templates: &Vec<Value>,
    ) -> (Vec<Uuid>, Vec<Value>) {
        let (list_ids, _) = setup_lists_default(client, session_response).await;
        let mut uuid_vec = Vec::<Uuid>::new();
        let mut value_vec = Vec::<Value>::new();
        for list_id in list_ids {
            for template in templates {
                let mut req = template.clone();
                req["list_id"] = Value::from(list_id.to_string());
                let res = client
                    .post("tasks")
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
                uuid_vec.push(response.id);
                value_vec.push(req);
            }
        }
        (uuid_vec, value_vec)
    }
}

pub mod extra {
    use reqwest::StatusCode;
    use serde_json::json;

    use super::utils::DEFAULT_TASKS_TEMPLATES;
    use crate::{
        api::{
            auth::email::utils::email_register_and_login_user_default,
            lists::utils::setup_lists_default,
        },
        commons,
    };

    #[rocket::async_test]
    pub async fn change_lists() {
        let client = commons::setup().await;
        let (session_response, _) = email_register_and_login_user_default(&client).await;

        // Create lists and task
        let (list_ids, _) = setup_lists_default(&client, &session_response).await;
        let task_id = utils::create_task(
            &client,
            &session_response,
            &DEFAULT_TASKS_TEMPLATES[0],
            list_ids[0],
        )
        .await;
        utils::assert_has_list(&client, &session_response, task_id, list_ids[0]).await;

        // Move task to different list
        let res = client
            .patch(&format!("tasks/{}", task_id))
            .json(&json!({
                "list_id": list_ids[1]
            }))
            .send()
            .await
            .expect("Expected a response");
        assert_eq!(res.status(), StatusCode::OK);
        utils::assert_has_list(&client, &session_response, task_id, list_ids[1]).await;
    }

    pub mod utils {
        use reqwest::StatusCode;
        use serde_json::Value;
        use uuid::Uuid;

        use super::super::parent_child::utils::get_item;
        use crate::{
            api::auth::email::utils::SessionResponse,
            commons::{http_client::HttpClient, utils::rest::PostResponse},
        };

        pub async fn create_task(
            client: &HttpClient,
            session_response: &SessionResponse,
            template: &Value,
            list_id: Uuid,
        ) -> Uuid {
            let mut req = template.clone();
            req["list_id"] = Value::from(list_id.to_string());
            let res = client
                .post("tasks")
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
            response.id
        }

        pub async fn assert_has_list(
            client: &HttpClient,
            session_response: &SessionResponse,
            child_id: Uuid,
            list_id: Uuid,
        ) {
            let child = get_item(&client, &session_response, child_id).await;
            assert_eq!(child.list_id, list_id, "Expected task to have list");
        }
    }
}
