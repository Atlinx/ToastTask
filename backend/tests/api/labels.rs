#![cfg(test)]

crate::test_crud! {
    model_path: "labels",
    model_plural: labels,
    get: {
        response_type: types::GetLabelResponse
    },
    post: {
        valid_item(_client, _session_response) {
            json!({
                "title": "Important",
                "color": "#ffa783",
            })
        },
        test_cases: {
            valid_0(_client, _session_response, StatusCode::CREATED) {
                json!({
                    "title": "Important",
                    "description": "Tasks that need to be done quickly.",
                    "color": "#ffa783",
                })
            },
            valid_1(_client, _session_response, StatusCode::CREATED) {
                json!({
                    "title": "Important",
                    "color": "#ffa783",
                })
            },

            invalid_0(_client, _session_response, StatusCode::UNPROCESSABLE_ENTITY) {
                json!({
                    "title": "Important",
                    "color": true,
                })
            },
            invalid_1(_client, _session_response, StatusCode::BAD_REQUEST) {
                json!({
                    "title": "Important",
                    "color": "sdfsdf",
                })
            }
        }
    },
    patch: {
        valid_changes(_client, _session_response) {
            json!({
                "title": "My updated label",
                "description": "This is an updated label",
                "color": "#444488",
            })
        },
        test_cases: {
            valid_0(_client, _session_response, StatusCode::OK) {
                json!({
                    "title": "My updated label",
                    "description": "This is an updated label",
                    "color": "#444488",
                })
            },
            valid_1(_client, _session_response, StatusCode::OK) {
                json!({
                    "title": "My updated list",
                    "description": null
                })
            },

            invalid_0(_client, _session_response, StatusCode::BAD_REQUEST) {
                json!({
                    "color": null,
                })
            },
            invalid_1(_client, _session_response, StatusCode::BAD_REQUEST) {
                json!({
                    "title": null,
                    "description": "This is an updated label",
                    "color": "#3849dfa"
                })
            }
        }
    },
    default_items: {
        json!({
            "title": "Important",
            "description": "Tasks that need to be done quickly.",
            "color": "#ffa783",
        }),
        json!({
            "title": "Backburner",
            "description": "Tasks to do later",
            "color": "#ba8778",
        }),
        json!({
            "title": "Homework",
            "color": "#89a783",
        }),
        json!({
            "title": "Birthday",
            "color": "#370073",
        })
    }
}

pub mod types {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct GetLabelResponse {
        pub id: Uuid,
        pub user_id: Uuid,
        pub title: String,
        pub description: Option<String>,
        pub color: String,
    }
}
