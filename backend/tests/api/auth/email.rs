#![cfg(test)]

use crate::commons;
use reqwest::StatusCode;
use rocket;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use self::utils::email_register_and_login_user_default;

#[allow(unused)]
#[derive(Deserialize)]
struct EmailLoginResponse {
    user_id: Uuid,
    session_token: Uuid,
}

#[rocket::async_test]
async fn email_register_valid() {
    let client = commons::setup().await;
    let res = client
        .post("/register/email")
        .json(&json!({
            "email": "johnsmith@gmail.com",
            "password": "mypassword",
            "username": "jonny"
        }))
        .send()
        .await
        .expect("Expected response");
    assert_eq!(res.status(), StatusCode::OK);
}

macro_rules! email_register {
    ($($name:ident: $input:expr,)*) => {
    $(
        #[rocket::async_test]
        async fn $name() {
            let (json, status) = $input;
            let client = commons::setup().await;
            let res = client
                .post("/register/email")
                .json(&json)
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), status);
        }
    )*
    }
}

email_register! {
    email_register_invalid_0: (json!({
        "email": "johnsmith@gmail.com",
        "password": "m",
        "username": "jonny"
    }), StatusCode::BAD_REQUEST),
    email_register_invalid_1: (json!({
        "email": "johnsmith@gmail.com",
        "username": "jonny"
    }), StatusCode::UNPROCESSABLE_ENTITY),
    email_register_invalid_2: (json!({
        "email": 5,
        "password": 20,
        "username": "jonny"
    }), StatusCode::UNPROCESSABLE_ENTITY),

    email_register_valid_0: (json!({
        "email": "johnsmith@gmail.com",
        "password": "mypassword",
        "username": "jonny"
    }), StatusCode::OK),
    email_register_valid_1: (json!({
        "email": "johnsmith@gmail.com",
        "password": "mypassword",
        "username": "jonny",
        "junk": 3
    }), StatusCode::OK),
}

#[rocket::async_test]
async fn email_login_valid_no_user() {
    let client = commons::setup().await;
    let res = client
        .post("/login/email")
        .json(&json!({
            "email": "johnsmith@gmail.com",
            "password": "mypassword"
        }))
        .send()
        .await
        .expect("Expected response");
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[rocket::async_test]
async fn email_login_valid_exists_user() {
    let client = commons::setup().await;
    email_register_and_login_user_default(&client).await;
}

#[rocket::async_test]
async fn email_login_valid_exists_user_amongst_multiple_users() {
    let client = commons::setup().await;
    let json_cred = json!({
        "email": "johnsmith@gmail.com",
        "password": "mypassword",
        "username": "johnny"
    });

    let register_reqs = [
        &json!({
            "email": "bobby@gmail.com",
            "password": "uncrackable",
            "username": "bobby"
        }),
        &json_cred,
        &json!({
            "email": "martha@gmail.com",
            "password": "sdfsdfio",
            "username": "mth"
        }),
        &json!({
            "email": "tony@gmail.com",
            "password": "imwalkingthere",
            "username": "big_tony83"
        }),
    ];
    for req_json in register_reqs {
        let res = client
            .post("/register/email")
            .json(&req_json)
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
    }
    let res = client
        .post("/login/email")
        .json(&json_cred)
        .send()
        .await
        .expect("Expected response");
    assert_eq!(res.status(), StatusCode::OK);
    res.json::<EmailLoginResponse>()
        .await
        .expect("Expect correct JSON response");
}

macro_rules! email_login_invalid {
    ($($name:ident: $input:expr,)*) => {
    $(
        #[rocket::async_test]
        async fn $name() {
            let (json, status) = $input;
            let client = commons::setup().await;
            let res = client
                .post("/login/email")
                .json(&json)
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), status);
        }
    )*
    }
}

email_login_invalid! {
    email_login_invalid_0: (json!({
        "email": "johnsm_gmail.com",
        "password": "mypassword"
    }), StatusCode::BAD_REQUEST),
    email_login_invalid_1: (json!({
        "password": "mypassword"
    }), StatusCode::UNPROCESSABLE_ENTITY),
    email_login_invalid_2: (json!({}), StatusCode::UNPROCESSABLE_ENTITY),
    email_login_invalid_3: (json!({
        "email": "johnsm_gmail.com",
    }), StatusCode::UNPROCESSABLE_ENTITY),
    email_login_invalid_4: (json!({
        "email": 530,
        "password": true
    }), StatusCode::UNPROCESSABLE_ENTITY),
    email_login_invalid_5: (json!({
        "email": "johnsmith@gmail.com",
        "password": "mypassword",
        "junk": 53
    }), StatusCode::UNAUTHORIZED),
}

pub mod utils {
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use uuid::Uuid;

    use crate::commons::http_client::HttpClient;

    #[derive(Deserialize)]
    pub struct SessionResponse {
        pub user_id: Uuid,
        pub session_token: Uuid,
    }

    #[derive(Serialize, Clone)]
    pub struct EmailLoginCredentials {
        pub email: String,
        pub password: String,
    }

    /// Registers and logs in a user, returning the session information for that user.
    pub async fn email_register_and_login_user_default(
        client: &HttpClient,
    ) -> (SessionResponse, EmailLoginCredentials) {
        email_register_and_login_user(client, "johnsmith").await
    }

    /// Registers and logs in a user, returning the session information for that user.
    pub async fn email_register_and_login_user(
        client: &HttpClient,
        username: &str,
    ) -> (SessionResponse, EmailLoginCredentials) {
        let credentials = EmailLoginCredentials {
            email: format!("{}@gmail.com", username),
            password: format!("mypassword{}", username).to_owned(),
        };
        let res = client
            .post("/register/email")
            .json(&json!({
                "email": credentials.email,
                "password": credentials.password,
                "username": username
            }))
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
        let resp = email_login_user(client, &credentials).await;
        (resp, credentials)
    }

    pub async fn email_login_user(
        client: &HttpClient,
        credentials: &EmailLoginCredentials,
    ) -> SessionResponse {
        let res = client
            .post("/login/email")
            .json(
                &serde_json::to_value(credentials.clone())
                    .expect("Expect credentials to serialize into json"),
            )
            .send()
            .await
            .expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
        let resp = res
            .json::<SessionResponse>()
            .await
            .expect("Expected login response json");
        resp
    }
}
