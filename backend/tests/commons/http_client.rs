#![allow(unused)]

use reqwest::{header, Error, Method, RequestBuilder, Response, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::api::auth::email::utils::SessionResponse;

use super::utils::rest::{GetAllResponse, PostResponse};

pub struct HttpClient {
    base_url: String,
    headers: header::HeaderMap,
    pub client: reqwest::Client,
}

impl HttpClient {
    pub fn new(base_url: &str, headers: header::HeaderMap) -> Result<Self, Error> {
        let client = reqwest::Client::builder().build()?;
        Ok(Self {
            base_url: base_url.to_owned(),
            headers,
            client,
        })
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.add_headers(self.client.get(self.get_path(path)))
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.add_headers(self.client.post(self.get_path(path)))
    }

    pub fn put(&self, path: &str) -> RequestBuilder {
        self.add_headers(self.client.put(self.get_path(path)))
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.add_headers(self.client.delete(self.get_path(path)))
    }

    pub fn patch(&self, path: &str) -> RequestBuilder {
        self.add_headers(self.client.patch(self.get_path(path)))
    }

    pub fn head(&self, path: &str) -> RequestBuilder {
        self.add_headers(self.client.head(self.get_path(path)))
    }

    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        self.add_headers(self.client.request(method, self.get_path(path)))
    }

    fn get_path(&self, path: &str) -> String {
        match path.contains("://") {
            true => path.to_owned(),
            false => {
                if !path.starts_with("/") {
                    format!("{}/{}", self.base_url, path)
                } else {
                    format!("{}{}", self.base_url, path)
                }
            }
        }
    }

    fn add_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        builder.headers(self.headers.clone())
    }
}

pub trait APIClient<'a, T> {
    fn api(&'a self) -> T
    where
        T: APIRequestBuilder<'a>;
}

#[rocket::async_trait]
pub trait APIRequestBuilder<'a> {
    fn auth(self, session_response: &'a SessionResponse) -> Self;
    fn path(self, path: &'a str) -> Self;
    async fn get<T: DeserializeOwned>(&self, id: Uuid) -> T;
    async fn get_all<T: DeserializeOwned>(&self) -> GetAllResponse<T>;
    async fn get_page<T: DeserializeOwned>(&self, page: u32, limit: u32) -> GetAllResponse<T>;
    async fn post(&self, body: Value) -> PostResponse;
    async fn patch(&self, id: Uuid, body: Value);
    async fn delete(&self, id: Uuid);
}

pub struct HttpClientAPIRequestBuilder<'a> {
    pub http_client: &'a HttpClient,
    pub path: Option<&'a str>,
    pub session_response: Option<&'a SessionResponse>,
}

#[rocket::async_trait]
impl<'a> APIRequestBuilder<'a> for HttpClientAPIRequestBuilder<'a> {
    fn auth(mut self, session_response: &'a SessionResponse) -> Self {
        self.session_response = Some(session_response);
        self
    }

    fn path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }

    async fn get<T: DeserializeOwned>(&self, item_id: Uuid) -> T {
        let path = self.path.expect("Expected path to be set.");
        let mut req = self.http_client.get(&format!("{}/{}", path, item_id));
        if let Some(session_response) = self.session_response {
            req = req.bearer_auth(session_response.session_token);
        }

        let res = req.send().await.expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
        let get_resp = res.json::<T>().await.expect("Expected json response");
        get_resp
    }

    async fn get_all<T: DeserializeOwned>(&self) -> GetAllResponse<T> {
        let path = self.path.expect("Expected path to be set.");
        let mut req = self.http_client.get(&format!("{}", path));
        if let Some(session_response) = self.session_response {
            req = req.bearer_auth(session_response.session_token);
        }

        let res = req.send().await.expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
        let get_resp = res
            .json::<GetAllResponse<T>>()
            .await
            .expect("Expected json response");
        get_resp
    }

    async fn get_page<T: DeserializeOwned>(&self, limit: u32, page: u32) -> GetAllResponse<T> {
        let path = self.path.expect("Expected path to be set.");
        let mut req = self
            .http_client
            .get(&format!("{}/?limit={}&page={}", path, limit, page));
        if let Some(session_response) = self.session_response {
            req = req.bearer_auth(session_response.session_token);
        }

        let res = req.send().await.expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
        let get_resp = res
            .json::<GetAllResponse<T>>()
            .await
            .expect("Expected json response");
        get_resp
    }

    async fn post(&self, body: Value) -> PostResponse {
        let path = self.path.expect("Expected path to be set.");
        let mut req = self.http_client.post(&format!("{}", path)).json(&body);
        if let Some(session_response) = self.session_response {
            req = req.bearer_auth(session_response.session_token);
        }

        let res = req.send().await.expect("Expected response");
        assert_eq!(res.status(), StatusCode::CREATED);
        res.json::<PostResponse>()
            .await
            .expect("Expected json response")
    }

    async fn patch(&self, id: Uuid, body: Value) {
        let path = self.path.expect("Expected path to be set.");
        let mut req = self
            .http_client
            .patch(&format!("{}/{}", path, id))
            .json(&body);
        if let Some(session_response) = self.session_response {
            req = req.bearer_auth(session_response.session_token);
        }

        let res = req.send().await.expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
    }

    async fn delete(&self, id: Uuid) {
        let path = self.path.expect("Expected path to be set.");
        let mut req = self.http_client.delete(&format!("{}/{}", path, id));
        if let Some(session_response) = self.session_response {
            req = req.bearer_auth(session_response.session_token);
        }

        let res = req.send().await.expect("Expected response");
        assert_eq!(res.status(), StatusCode::OK);
    }
}

impl<'a> APIClient<'a, HttpClientAPIRequestBuilder<'a>> for HttpClient {
    fn api(&'a self) -> HttpClientAPIRequestBuilder<'a> {
        HttpClientAPIRequestBuilder::<'a> {
            http_client: &self,
            path: None,
            session_response: None,
        }
    }
}
