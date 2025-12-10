use std::net::IpAddr;

use actix_web::{HttpRequest, web};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{self, api_response::ApiResponse};

#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    expires_in: String,
}

#[derive(Deserialize)]
struct StkPushResponse {
    ResponseCode: String,
    ResponseDescription: String,
    MerchantRequestID: String,
    CheckoutRequestID: String,
    ResultCode: String,
    ResultDesc: String,
}

pub struct MpesaClient {
    pub client: Client,
    pub base_url: String,
    pub consumer_key: String,
    pub consumer_secret: String,
}

impl MpesaClient {
    pub fn new() -> Self {
        let base_url = (utils::constants::MPESA_BASE_URL).clone();
        let consumer_key = (utils::constants::MPESA_CONSUMER_KEY).clone();
        let consumer_secret = (utils::constants::MPESA_CONSUMER_SECRET).clone();
        
        Self {
            client: Client::new(),
            base_url,
            consumer_key,
            consumer_secret,
        }
    }

    pub async fn get_access_token(&self) -> Result<String, reqwest::Error> {
        let url = format!("{}/oauth/v1/generate?grant_type=client_credentials", self.base_url);

        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.consumer_key, Some(&self.consumer_secret))
            .send()
            .await?
            .json::<AccessTokenResponse>()
            .await?;

        Ok(resp.access_token)
    }

    pub async fn stk_push<T: Serialize + ?Sized>(
        &self,
        payload: &T,
    ) -> Result<StkPushResponse, reqwest::Error> {
        let token = self.get_access_token().await?;

        let url = format!("{}/mpesa/stkpush/v1/processrequest", self.base_url);

        let resp = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(payload)
            .send()
            .await?
            .json::<StkPushResponse>()
            .await?;

        Ok(resp)
    }
}

const WHITELISTED_IPS: &[&str] = &[
    "196.201.214.200",
    "196.201.214.206",
    "196.201.213.114",
    "196.201.214.207",
    "196.201.214.208",
    "196.201.213.44",
    "196.201.212.127",
    "196.201.212.138",
    "196.201.212.129",
    "196.201.212.136",
    "196.201.212.74",
    "196.201.212.69",
];

fn is_ip_whitelisted(ip: &IpAddr) -> bool {
    WHITELISTED_IPS.iter().any(|&whitelisted| ip.to_string() == whitelisted)
}

pub async fn mpesa_callback(
    req: HttpRequest,
    body: web::Bytes,
) -> Result<ApiResponse, ApiResponse> {
    // Get client IP
    let peer_ip = req
        .connection_info()
        .realip_remote_addr()
        .and_then(|addr| addr.split(':').next())
        .and_then(|ip_str| ip_str.parse::<IpAddr>().ok());

    if let Some(ip) = peer_ip {
        if !is_ip_whitelisted(&ip) {
            println!("Blocked IP: {}", ip);
            return Err(ApiResponse::new(200, json!({"message": "IP not allowed"})))
        }
    } else {
        println!("Could not determine client IP");
        return Err(ApiResponse::new(200, json!({"message": "IP not allowed"})))
    }

    let payload = std::str::from_utf8(&body).unwrap_or_default();
    println!("Received Mpesa callback: {}", payload);

    Ok(ApiResponse::new(200, json!({
        "ResultCode": 0,
        "ResultDesc": "Accepted"
    })))
}
