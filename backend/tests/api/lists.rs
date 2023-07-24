#![cfg(test)]

crate::test_crud! {
    model_path: "lists",
    model_plural: lists,
    get: {
        response_type: types::GetListResponse
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
        }),
        json!({
            "title": "School list",
            "color": "#89a783",
        }),
        json!({
            "title": "Work list",
            "description": "List for work related stuff.",
            "color": "#370073",
        }),
        json!({
            "title": "Goals list",
            "description": "List for big goals.",
            "color": "#ff3872",
        })
    }
}

// TODO NOW: Debug api::lists::parent_child::reparent

crate::test_tree_crud! {
    model_path: "lists",
    response_type: types::GetListResponse,
    valid_item: json!({
        "title": "Grocery list",
        "color": "#ffa783",
    }),
    rud_setup: utils::rud_setup
}

pub mod types {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct GetListResponse {
        pub id: Uuid,
        pub user_id: Uuid,
        pub title: String,
        pub description: Option<String>,
        pub color: String,
        pub parent_id: Option<Uuid>,
        pub child_ids: Vec<Uuid>,
    }
}
