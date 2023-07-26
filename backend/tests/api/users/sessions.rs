#![cfg(test)]

crate::test_get! {
  model_path: "users/me/sessions",
  response_type: types::GetSessionResponse,
  rud_setup: utils::rud_setup
}

crate::test_delete! {
    model_path: "users/me/sessions",
    rud_setup: utils::rud_setup
}

pub mod types {
    use ipnetwork::IpNetwork;
    use serde::{Deserialize, Serialize};
    use time::PrimitiveDateTime;
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct GetSessionResponse {
        pub id: Uuid,
        pub ip: IpNetwork,
        pub platform: String,
        pub user_agent: String,
        pub created_at: PrimitiveDateTime,
        pub expire_at: PrimitiveDateTime,
    }
}

pub mod utils {
    use serde_json::{json, Value};
    use uuid::Uuid;

    use crate::{
        api::auth::email::utils::{
            email_login_user, email_register_and_login_user, email_register_and_login_user_default,
            EmailLoginCredentials, SessionResponse,
        },
        commons::http_client::HttpClient,
    };

    pub async fn rud_setup(client: &HttpClient) -> (SessionResponse, Vec<Uuid>, Vec<Value>) {
        // Other user's data, which should be irrelevant
        for i in 0..10 {
            let (_, credentials) =
                email_register_and_login_user(client, &format!("alex{}", i)).await;
            setup_sessions(client, &credentials).await;
        }

        let (session_response, credentials) = email_register_and_login_user_default(client).await;
        let (item_ids, items) = setup_sessions(client, &credentials).await;

        (session_response, item_ids, items)
    }

    pub async fn setup_sessions(
        client: &HttpClient,
        credentials: &EmailLoginCredentials,
    ) -> (Vec<Uuid>, Vec<Value>) {
        let mut session_ids = Vec::<Uuid>::new();
        let mut session_responses = Vec::<Value>::new();
        for _ in 0..10 {
            let resp = email_login_user(client, credentials).await;
            session_ids.push(resp.session_token);
            session_responses.push(json!({
              "user_id": resp.user_id
            }));
        }
        (session_ids, session_responses)
    }
}
