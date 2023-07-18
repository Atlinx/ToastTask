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
            false => format!("{}{}", self.base_url, path),
        }
    }

    fn add_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        builder.headers(self.headers.clone())
    }
}
