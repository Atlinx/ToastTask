#![cfg(test)]

use crate::commons;
use reqwest::StatusCode;
use rocket;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[allow(unused)]
#[derive(Deserialize)]
struct EmailLoginResponse {
    user_id: Uuid,
    session_token: Uuid,
}

#[rocket::async_test]
async fn email_register_valid() {
    let (client, app) = commons::setup().await;
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
    app.shutdown().await;
}

macro_rules! email_register {
    ($($name:ident: $input:expr,)*) => {
    $(
        #[rocket::async_test]
        async fn $name() {
            let (json, status) = $input;
            let (client, app) = commons::setup().await;
            let res = client
                .post("/register/email")
                .json(&json)
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), status);
            app.shutdown().await;
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
    let (client, app) = commons::setup().await;
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
    app.shutdown().await;
}

#[rocket::async_test]
async fn email_login_valid_exists_user() {
    let (client, app) = commons::setup().await;
    let json_cred = &json!({
        "email": "johnsmith@gmail.com",
        "password": "mypassword",
        "username": "johnny"
    });
    let res = client
        .post("/register/email")
        .json(&json_cred)
        .send()
        .await
        .expect("Expected response");
    assert_eq!(res.status(), StatusCode::OK);
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
    app.shutdown().await;
}

#[rocket::async_test]
async fn email_login_valid_exists_user_amongst_multiple_users() {
    let (client, app) = commons::setup().await;
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
    app.shutdown().await;
}

macro_rules! email_login_invalid {
    ($($name:ident: $input:expr,)*) => {
    $(
        #[rocket::async_test]
        async fn $name() {
            let (json, status) = $input;
            let (client, app) = commons::setup().await;
            let res = client
                .post("/login/email")
                .json(&json)
                .send()
                .await
                .expect("Expected response");
            assert_eq!(res.status(), status);
            app.shutdown().await;
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
