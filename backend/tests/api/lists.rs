#![cfg(test)]

crate::test_crud! {
    model_plural: lists,
    get: {
        response_type: GetListResponse
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
            "title": "Grocery list",
            "description": "List of groceries for next week's event.",
            "color": "#ffa783",
        }),
        json!({
            "title": "Todo list",
            "description": "Dail for next week's event.",
            "color": "#ba8778",
        }),
        json!({
            "title": "Homework list",
            "color": "#89a783",
        }),
        json!({
            "title": "Birthday list",
            "color": "#370073",
        })
    }
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetListResponse {
    id: Uuid,
    user_id: Uuid,
    title: String,
    description: Option<String>,
    color: String,
}
