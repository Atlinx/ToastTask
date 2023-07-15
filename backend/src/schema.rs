// @generated automatically by Diesel CLI.

diesel::table! {
    discord_user_login (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        client_id -> Text,
    }
}

diesel::table! {
    labels (id) {
        id -> Int4,
        title -> Text,
        description -> Text,
        #[max_length = 7]
        color -> Varchar,
    }
}

diesel::table! {
    lists (id) {
        id -> Int4,
        title -> Text,
        description -> Nullable<Text>,
        #[max_length = 7]
        color -> Varchar,
        user_id -> Nullable<Int4>,
    }
}

diesel::table! {
    lists_hierarchy (child_list_id, parent_list_id) {
        child_list_id -> Int4,
        parent_list_id -> Int4,
    }
}

diesel::table! {
    task_labels (task_id, label_id) {
        task_id -> Int4,
        label_id -> Int4,
    }
}

diesel::table! {
    tasks (id) {
        id -> Int4,
        created_date -> Timestamp,
        edited_date -> Timestamp,
        due_date -> Timestamp,
        due_text -> Text,
        completed -> Bool,
        title -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    tasks_hierarchy (child_task_id, parent_task_id) {
        child_task_id -> Int4,
        parent_task_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
    }
}

diesel::joinable!(discord_user_login -> users (user_id));
diesel::joinable!(lists -> users (user_id));
diesel::joinable!(task_labels -> labels (label_id));
diesel::joinable!(task_labels -> tasks (task_id));

diesel::allow_tables_to_appear_in_same_query!(
    discord_user_login,
    labels,
    lists,
    lists_hierarchy,
    task_labels,
    tasks,
    tasks_hierarchy,
    users,
);
