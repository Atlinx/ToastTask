#![allow(unused)]

use reqwest::{header, Error, Method, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

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
        let url = format!("{}{}", self.base_url, path);
        self.client.get(&url).headers(self.headers.clone())
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client.post(&url).headers(self.headers.clone())
    }

    pub fn put(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client.put(&url).headers(self.headers.clone())
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client.delete(&url).headers(self.headers.clone())
    }

    pub fn patch(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client.patch(&url).headers(self.headers.clone())
    }

    pub fn head(&self, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client.head(&url).headers(self.headers.clone())
    }

    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .request(method, &url)
            .headers(self.headers.clone())
    }
}
