use actix_web::HttpRequest;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

use crate::utils;

pub fn get_bearer_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

pub struct ApiClient {
    pub client: Client,
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
}

impl ApiClient {
    pub fn new() -> Self {
        let base_url = (utils::constants::SSO_BASE_URL).clone();
        let client_id = (utils::constants::SSO_CLIENT_ID).clone();
        let client_secret = (utils::constants::SSO_CLIENT_SECRET).clone();

        Self {
            client: Client::new(),
            base_url,
            client_id,
            client_secret,
        }
    }

    pub async fn call<T: DeserializeOwned, P: Serialize>(
        &self,
        path: &str,
        req: &HttpRequest,
        payload: Option<&P>,
        method: reqwest::Method,
    ) -> Result<T, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);

        let mut request = self
            .client
            .request(method, &url)
            .header("Client-ID", &self.client_id)
            .header("Client-Secret", &self.client_secret);

        if let Some(token) = get_bearer_token(&req) {
            request = request.bearer_auth(token);
        }

        if let Some(body) = payload {
            request = request.json(body);
        }

        request.send().await?.json::<T>().await
    }
}
