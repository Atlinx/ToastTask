#![cfg(test)]

crate::test_crud! {
    model_path: "tasks",
    model_plural: tasks,
    get: {
        response_type: types::GetTaskResponse
    },
    post: {
        valid_item(_client, _session_response) {
            json!({
                "title": "Get groceries task",
                "due_at": "2023-10-19T10:23:00.000000000Z",
                "due_text": "Next Monday"
            })
        },
        test_cases: {
            valid_0(client, session_response, StatusCode::CREATED) {
                use crate::api::lists::utils::setup_lists_default;
                let (list_ids, _) = setup_lists_default(client, session_response).await;
                json!({
                    "list_id": list_ids[0],
                    "title": "Get groceries task",
                    "description": "Task for getting groceries for next week's event.",
                    "due_at": "2023-10-19T10:23:00.000000000Z",
                    "due_text": "Next Monday"
                })
            },
            valid_1(client, session_response, StatusCode::CREATED) {
                use crate::api::lists::utils::setup_lists_default;
                let (list_ids, _) = setup_lists_default(client, session_response).await;
                json!({
                    "list_id": list_ids[0],
                    "title": "Get groceries task",
                    "due_at": "2023-10-19T10:23:00.000000000Z",
                    "due_text": "Next Monday"
                })
            },

            invalid_0(client, session_response, StatusCode::UNPROCESSABLE_ENTITY) {
                use crate::api::lists::utils::setup_lists_default;
                let (list_ids, _) = setup_lists_default(client, session_response).await;
                json!({
                    "id": list_ids[0],
                    "title": "Get groceries task",
                    "description": true,
                })
            },
            invalid_1(_client, _session_response, StatusCode::UNPROCESSABLE_ENTITY) {
                json!({})
            }
        }
    },
    patch: {
        valid_changes(_client, _session_response) {
            json!({
                "title": "My updated task",
                "description": "This is an updated task",
                "due_at": "2023-10-19T10:23:00.000000000Z",
                "due_text": "Next Monday"
            })
        },
        test_cases: {
            valid_0(_client, _session_response, StatusCode::OK) {
                json!({
                    "title": "My updated task",
                    "description": "This is an updated list",
                    "due_at": "2023-10-19T10:23:00.000000000Z",
                    "due_text": "Next Monday"
                })
            },
            valid_1(_client, _session_response, StatusCode::OK) {
                json!({
                    "title": "My updated task",
                    "description": null,
                    "due_at": "2023-10-19T10:23:00.000000000Z",
                    "due_text": "Next Monday"
                })
            },

            invalid_0(_client, _session_response, StatusCode::BAD_REQUEST) {
                json!({})
            },
            invalid_1(_client, _session_response, StatusCode::BAD_REQUEST) {
                json!({
                    "title": null,
                    "description": "This is an updated task"
                })
            }
        }
    },
    default_items: {
        json!({
            "due_at": "2023-10-19T10:23:00.000000000Z",
            "due_text": "Next Monday",
            "title": "Go shopping with friends"
        }),
        json!({
            "due_at": "2023-10-29T13:10:00.000000000Z",
            "due_text": "Every Thursday",
            "completed": true,
            "title": "Taking out the garbage"
        }),
        json!({
            "due_at": "2023-10-19T10:00:00.000000000Z",
            "due_text": "Due thursday",
            "completed": true,
            "title": "Attend meeting"
        }),
        json!({
            "due_at": "2023-10-30T15:30:00.000000000Z",
            "due_text": "Due Friday",
            "completed": false,
            "title": "Attend office hours"
        }),
        json!({
            "due_at": "2023-11-03T08:00:00.000000000Z",
            "due_text": "Every Thursday",
            "title": "Call parents"
        }),
        json!({
            "due_at": "2023-11-05T10:23:00.000000000Z",
            "due_text": "November 11",
            "completed": false,
            "title": "Meet with freinds"
        }),
        json!({
            "due_at": "2023-11-06T20:30:00.000000000Z",
            "due_text": "November 11",
            "title": "Prepare birthday gift"
        })
    },
    rud_setup_items(client, session_response, templates) {
        use crate::api::lists::utils::setup_lists_default;
        use crate::commons::utils::rest::PostResponse;
        use reqwest::StatusCode;

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

crate::test_tree_crud! {
    model_path: "tasks",
    response_type: types::GetTaskResponse,
    valid_item(client, session_response) {
        use crate::api::lists::utils::setup_lists_default;
        let (list_ids, _) = setup_lists_default(client, session_response).await;

        json!({
            "list_id": list_ids[0],
            "title": "Get groceries task",
            "description": "Get groceries task",
            "due_at": "2023-10-19T10:23:00.000000000Z",
            "due_text": "Next Monday"
        })
    },
    rud_setup: utils::rud_setup
}

pub mod types {
    use crate::commons::utils::serde::{
        primitive_date_iso_deserialize, primitive_date_iso_serialize,
    };
    use serde::{Deserialize, Serialize};
    use time::PrimitiveDateTime;
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct GetTaskResponse {
        pub id: Uuid,
        pub parent_id: Option<Uuid>,
        pub list_id: Uuid,
        #[serde(
            serialize_with = "primitive_date_iso_serialize",
            deserialize_with = "primitive_date_iso_deserialize"
        )]
        pub created_at: PrimitiveDateTime,
        #[serde(
            serialize_with = "primitive_date_iso_serialize",
            deserialize_with = "primitive_date_iso_deserialize"
        )]
        pub updated_at: PrimitiveDateTime,
        #[serde(
            serialize_with = "primitive_date_iso_serialize",
            deserialize_with = "primitive_date_iso_deserialize"
        )]
        pub due_at: PrimitiveDateTime,
        pub due_text: String,
        pub completed: bool,
        pub title: String,
        pub description: Option<String>,
        pub child_ids: Vec<Uuid>,
        pub label_ids: Vec<Uuid>,
    }
}

pub mod lists {
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

        use super::super::tree::utils::get_item;
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

pub mod labels {
    use super::utils::setup_tasks_default;
    use crate::{
        api::{
            auth::email::utils::email_register_and_login_user_default,
            labels::utils::setup_labels_default, tasks::types::GetTaskResponse,
        },
        commons::{
            self,
            http_client::{APIClient, APIRequestBuilder},
        },
    };

    #[rocket::async_test]
    pub async fn change_labels() {
        let client = commons::setup().await;
        let (session_response, _) = email_register_and_login_user_default(&client).await;

        println!("change_labels");
        println!("  1");
        let (task_ids, _) = setup_tasks_default(&client, &session_response).await;
        let (label_ids, _) = setup_labels_default(&client, &session_response).await;

        println!("  2");
        utils::assert_missing_label(&client, &session_response, task_ids[0], label_ids[1]).await;
        utils::assert_missing_label(&client, &session_response, task_ids[0], label_ids[2]).await;

        println!("  3");
        utils::add_label(&client, &session_response, task_ids[0], label_ids[1]).await;
        utils::add_label(&client, &session_response, task_ids[0], label_ids[2]).await;

        println!("  4");
        utils::assert_has_label(&client, &session_response, task_ids[0], label_ids[1]).await;
        utils::assert_has_label(&client, &session_response, task_ids[0], label_ids[2]).await;

        println!("  5");
        utils::delete_label(&client, &session_response, task_ids[0], label_ids[1]).await;

        println!("  6");
        utils::assert_missing_label(&client, &session_response, task_ids[0], label_ids[1]).await;
        utils::assert_has_label(&client, &session_response, task_ids[0], label_ids[2]).await;
        println!("  7");
    }

    #[rocket::async_test]
    pub async fn get_labels() {
        let client = commons::setup().await;
        let (session_response, _) = email_register_and_login_user_default(&client).await;

        let (task_ids, _) = setup_tasks_default(&client, &session_response).await;
        let (label_ids, _) = setup_labels_default(&client, &session_response).await;

        let item = client
            .api()
            .path("tasks")
            .auth(&session_response)
            .get::<GetTaskResponse>(task_ids[0])
            .await;
        assert!(
            utils::set_eq(&item.child_ids, &vec![]),
            "Expected child_ids to have no labels"
        );

        utils::add_label(&client, &session_response, task_ids[0], label_ids[1]).await;
        utils::add_label(&client, &session_response, task_ids[0], label_ids[3]).await;
        utils::add_label(&client, &session_response, task_ids[0], label_ids[4]).await;

        let item = get_item(&client, &session_response, task_ids[0]).await;
        assert!(
            utils::set_eq(
                &item.child_ids,
                &vec![task_ids[0], task_ids[3], task_ids[4]]
            ),
            "Expected child_ids to have correct labels"
        );
    }

    pub mod utils {
        use reqwest::StatusCode;
        use serde_json::json;
        use std::{collections::HashSet, hash::Hash};
        use uuid::Uuid;

        use crate::{
            api::{auth::email::utils::SessionResponse, tasks::types::GetTaskResponse},
            commons::http_client::{APIClient, APIRequestBuilder, HttpClient},
        };

        pub fn set_eq<T>(a: &[T], b: &[T]) -> bool
        where
            T: Eq + Hash,
        {
            let a: HashSet<_> = a.iter().collect();
            let b: HashSet<_> = b.iter().collect();

            a == b
        }

        pub async fn add_label(
            client: &HttpClient,
            session_response: &SessionResponse,
            task_id: Uuid,
            label_id: Uuid,
        ) {
            let res = client
                .post(&format!("tasks/{}/labels", task_id))
                .bearer_auth(session_response.session_token)
                .json(&json!({
                    "id": label_id
                }))
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), StatusCode::OK);
        }

        pub async fn delete_label(
            client: &HttpClient,
            session_response: &SessionResponse,
            task_id: Uuid,
            label_id: Uuid,
        ) {
            let res = client
                .delete(&format!("tasks/{}/labels/{}", task_id, label_id))
                .bearer_auth(session_response.session_token)
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), StatusCode::OK);
        }

        pub async fn assert_missing_label(
            client: &HttpClient,
            session_response: &SessionResponse,
            task_id: Uuid,
            label_id: Uuid,
        ) {
            let res = client
                .api()
                .path("tasks")
                .auth(session_response)
                .get::<GetTaskResponse>(task_id)
                .await;
            assert!(
                !res.label_ids.contains(&label_id),
                "Expected task not to have label: {}",
                label_id
            );
        }

        pub async fn assert_has_label(
            client: &HttpClient,
            session_response: &SessionResponse,
            task_id: Uuid,
            label_id: Uuid,
        ) {
            let res = client
                .api()
                .path("tasks")
                .auth(session_response)
                .get::<GetTaskResponse>(task_id)
                .await;
            assert!(
                res.label_ids.contains(&label_id),
                "Expected task to have label: {}",
                label_id
            );
        }
    }
}
