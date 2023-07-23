#![cfg(test)]

crate::test_crud! {
    model_path: "labels",
    model_plural: labels,
    get: {
        response_type: types::GetLabelResponse
    },
    post: {
        valid_item: json!({
            "title": "Important",
            "color": "#ffa783",
        }),
        test_cases: {
            valid_0: (json!({
                "title": "Important",
                "description": "Tasks that need to be done quickly.",
                "color": "#ffa783",
            }), StatusCode::CREATED),
            valid_1: (json!({
                "title": "Important",
                "color": "#ffa783",
            }), StatusCode::CREATED),

            invalid_0: (json!({
                "title": "Important",
                "color": true,
            }), StatusCode::UNPROCESSABLE_ENTITY),
            invalid_1: (json!({
                "title": "Important",
                "color": "sdfsdf",
            }), StatusCode::BAD_REQUEST),
        }
    },
    patch: {
        valid_changes: json!({
            "title": "My updated label",
            "description": "This is an updated label",
            "color": "#444488",
        }),
        test_cases: {
            valid_0: (json!({
                "title": "My updated label",
                "description": "This is an updated label",
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
                "description": "This is an updated label",
                "color": "#3849dfa"
            }), StatusCode::BAD_REQUEST),
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
