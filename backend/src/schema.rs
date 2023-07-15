// @generated automatically by Diesel CLI.

diesel::table! {
    discord_user_login (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        client_id -> Text,
    }
}

diesel::table! {
    labels (id) {
        id -> Uuid,
        title -> Text,
        description -> Text,
        #[max_length = 7]
        color -> Varchar,
    }
}

diesel::table! {
    list_relations (child_list_id, parent_list_id) {
        child_list_id -> Uuid,
        parent_list_id -> Uuid,
    }
}

diesel::table! {
    lists (id) {
        id -> Uuid,
        title -> Text,
        description -> Nullable<Text>,
        #[max_length = 7]
        color -> Varchar,
        user_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        ip -> Cidr,
        created_date -> Timestamp,
        expire_date -> Timestamp,
        user_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    task_labels (task_id, label_id) {
        task_id -> Uuid,
        label_id -> Uuid,
    }
}

diesel::table! {
    task_relations (child_task_id, parent_task_id) {
        child_task_id -> Uuid,
        parent_task_id -> Uuid,
    }
}

diesel::table! {
    tasks (id) {
        id -> Uuid,
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
    users (id) {
        id -> Uuid,
        username -> Text,
    }
}

diesel::joinable!(discord_user_login -> users (user_id));
diesel::joinable!(lists -> users (user_id));
diesel::joinable!(sessions -> users (user_id));
diesel::joinable!(task_labels -> labels (label_id));
diesel::joinable!(task_labels -> tasks (task_id));

diesel::allow_tables_to_appear_in_same_query!(
    discord_user_login,
    labels,
    list_relations,
    lists,
    sessions,
    task_labels,
    task_relations,
    tasks,
    users,
);
