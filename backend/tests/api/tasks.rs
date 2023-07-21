#![cfg(test)]

crate::test_crud! {
    model_plural: tasks,
    get: {
        response_type: GetTaskResponse
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
}

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetTaskResponse {
    id: Uuid,
    list_id: Uuid,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    due_at: NaiveDateTime,
    due_text: String,
    completed: bool,
    title: String,
    description: Option<String>,
}

// TODO LATER: Make tests for tasks & labels
//
//  Consider refactoring test_crud_utils macro
//  to allow for customization of setup function,
//  since the setup of tasks depends on the setup
//  of various lists
//  
//  Consider letting rud_setup return a generic struct,
//  with some mandatory fields required by the macro,
//  but also additional fields that can then be used
//  in the various test_case and other expressions
//  within the test_crud macro.
//
//  Consider using Tauri for frontend, to make this app
//  cross platform. 
pub mod utils {
    use serde_json::json;

    use crate::api::auth::email::utils::SessionResponse;

    pub static DEFAULT_TASKS_TEMPLATE: Lazy<Vec<Value>> = Lazy::new(|| vec![
        json!({
           due_at: "2023-10-19 10:23:54"
           due_text: "Next Monday"
           title: "Go shopping with friends"
        }),
        json!({
            due_at: "2023-10-19 10:23:54"
            due_text: "Every Thursday",
            completed: true,
            title: "Taking out the garbage"
        }),
        json!({
            due_at: "2023-10-19 10:23:54"
            due_text: "Due thursday",
            completed: true,
            title: "Taking out the garbage"
        })
    ]);

    pub async fn rud_setup(
        client: &HttpClient,
    ) -> (SessionResponse, Vec<Uuid>, &Vec<serde_json::Value>) {
        // Other user's data, which should be irrelevant
        for i in 0..10 {
            let session_response =
                email_register_and_login_user(client, &format!("alex{}", i)).await;
            setup_tasks_default>](client, &session_response).await;
        }

        let session_response = email_register_and_login_user_default(client).await;
        let (item_ids, items) = setup_tasks_default(client, &session_response).await;

        (session_response, item_ids, &items)
    }

    pub async fn setup_tasks_default(client: &HttpClient, session_response: &SessionResponse) -> (Vec<Uuid>, Vec<serde_json::Value>) {
        
    }

    pub async fn setup_tasks(client)
}