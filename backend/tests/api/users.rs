#![cfg(test)]

pub mod get {
    use assert_json_diff::assert_json_include;
    use reqwest::StatusCode;
    use serde_json::Value;

    use super::utils::rud_setup;
    use crate::commons;

    #[rocket::async_test]
    async fn get_me_unauth() {
        let client = commons::setup().await;
        let _ = rud_setup(&client).await;
        let res = client
            .get("users/me")
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[rocket::async_test]
    async fn get_me() {
        let client = commons::setup().await;
        let (session_response, user_json) = rud_setup(&client).await;
        let res = client
            .get("users/me")
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let json_res = res.json::<Value>().await.expect("Exprected json response");
        assert_json_include!(
            actual: json_res,
            expected: user_json
        )
    }
}

pub mod patch {
    use assert_json_diff::assert_json_include;
    use reqwest::StatusCode;
    use serde_json::{json, Value};

    use super::utils::rud_setup;
    use crate::commons;

    #[rocket::async_test]
    async fn patch_me_unauth() {
        let client = commons::setup().await;
        let _ = rud_setup(&client).await;
        let res = client
            .patch("users/me")
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[rocket::async_test]
    async fn patch_me() {
        let client = commons::setup().await;
        let (session_response, _) = rud_setup(&client).await;
        let changes = json!({
            "username": "new_name"
        });
        let res = client
            .patch("users/me")
            .bearer_auth(session_response.session_token)
            .json(&changes)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
        let refetch_item = client
            .get("users/me")
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response")
            .json::<Value>()
            .await
            .expect("Expected json response");
        assert_json_include!(
            actual: refetch_item,
            expected: changes
        );
    }
}

pub mod delete {
    use reqwest::StatusCode;

    use super::utils::rud_setup;
    use crate::commons;

    #[rocket::async_test]
    async fn delete_me_unauth() {
        let client = commons::setup().await;
        let _ = rud_setup(&client).await;
        let res = client
            .delete("users/me")
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[rocket::async_test]
    async fn delete_me() {
        let client = commons::setup().await;
        let (session_response, _) = rud_setup(&client).await;
        let confirm_exists_res = client
            .get("users/me")
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(confirm_exists_res.status(), StatusCode::OK);
        let res = client
            .delete("users/me")
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let confirm_deleted_res = client
            .get("users/me")
            .bearer_auth(session_response.session_token)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(confirm_deleted_res.status(), StatusCode::UNAUTHORIZED);
        // When deleted our session should have been invalidated as well
    }
}

pub mod types {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use crate::api::sessions::types::GetSessionResponse;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GetUserResponse {
        id: Uuid,
        username: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
        discord_login: Option<GetDiscordUserLoginResponse>,
        email_login: Option<GetEmailUserLoginResponse>,
        sessions: Vec<GetSessionResponse>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GetDiscordUserLoginResponse {
        client_id: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GetEmailUserLoginResponse {
        email: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PatchUserResponse {
        username: String,
    }
}

pub mod utils {
    use serde_json::{json, Value};

    use crate::{
        api::{
            auth::email::utils::{email_register_and_login_user, SessionResponse},
            sessions::utils::setup_sessions,
        },
        commons::http_client::HttpClient,
    };

    pub async fn rud_setup(client: &HttpClient) -> (SessionResponse, Value) {
        // Other user's data, which should be irrelevant
        for i in 0..10 {
            setup_user(client, &format!("alex{}", i)).await;
        }

        setup_user(client, "joe").await
    }

    pub async fn setup_user(client: &HttpClient, username: &str) -> (SessionResponse, Value) {
        let (session_response, credentials) =
            email_register_and_login_user(client, &username).await;
        let (_, sessions_json) = setup_sessions(client, &credentials).await;
        let user_json = json!({
            "id": session_response.user_id,
            "username": username,
            "sessions": sessions_json,
            "email_login": {
                "email": credentials.email
            },
        });

        (session_response, user_json)
    }
}
