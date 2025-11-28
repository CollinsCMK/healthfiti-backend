use actix_rt::spawn;
use actix_web::web;
use lettre::{
    Message, SmtpTransport, Transport, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use log;
use redis::AsyncCommands;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::utils::{api_response::ApiResponse, constants};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Email {
        to: String,
        subject: String,
        html: String,
    },
    SMS {
        phone_number: String,
        message: String,
    },
}

pub struct MessageQueue {
    redis_client: redis::Client,
}

impl MessageQueue {
    pub fn new(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
        MessageQueue {
            redis_client: client,
        }
    }

    pub async fn send_message(&self, message: MessageType) -> Result<(), String> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;
        let message_json = serde_json::to_string(&message).map_err(|e| e.to_string())?;

        conn.rpush("message_queue", message_json)
            .await
            .map_err(|e| format!("Failed to push to Redis: {}", e))
    }
}

pub fn init_message_queue(redis_url: &str) -> web::Data<MessageQueue> {
    let queue = MessageQueue::new(redis_url);
    let redis_url_owned = redis_url.to_string(); // Convert &str to String

    spawn(async move {
        process_messages(&redis_url_owned).await; // Move adminship
    });

    web::Data::new(queue)
}

async fn process_messages(redis_url: &str) {
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    log::info!("Message processor started");

    loop {
        if let Some(message) = fetch_message(&mut conn).await {
            match serde_json::from_str::<MessageType>(&message) {
                Ok(parsed_message) => match parsed_message {
                    MessageType::Email { to, subject, html } => {
                        if let Err(err) = send_email(to, subject, html).await {
                            log::error!("Failed to send email: {:?}", err);
                        }
                    }
                    MessageType::SMS {
                        phone_number,
                        message,
                    } => {
                        log::info!("Processing SMS message to: {}", phone_number);
                        if let Err(err) = send_sms(phone_number, message).await {
                            log::error!("Failed to send SMS: {:?}", err);
                        } else {
                            log::info!("SMS sent successfully");
                        }
                    }
                },
                Err(err) => log::error!("Failed to parse message: {}", err),
            }
        }
    }
}

async fn fetch_message(conn: &mut redis::aio::MultiplexedConnection) -> Option<String> {
    let result: Result<Option<(String, String)>, _> = conn.blpop("message_queue", 0.0).await;
    match result {
        Ok(Some((_queue, message))) => Some(message),
        _ => None,
    }
}

async fn send_email(to: String, subject: String, html: String) -> Result<ApiResponse, ApiResponse> {
    let email_from = (*constants::MAIL_FROM_ADDRESS).clone();
    let email_username = (*constants::MAIL_USERNAME).clone();
    let email_password = (*constants::MAIL_PASSWORD).clone();
    let email_host = (*constants::MAIL_HOST).clone();

    let email = match Message::builder()
        .from(email_from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html)
    {
        Ok(email) => email,
        Err(e) => {
            return Err(ApiResponse::new(
                500,
                json!({ "message": format!("Failed to build email: {}", e) }),
            ));
        }
    };

    let creds = Credentials::new(email_username.clone(), email_password.clone());
    let mailer = match SmtpTransport::relay(&email_host) {
        Ok(transport) => transport.credentials(creds).build(),
        Err(e) => {
            return Err(ApiResponse::new(
                500,
                json!({ "message": format!("SMTP transport creation failed: {}", e) }),
            ));
        }
    };

    match mailer.send(&email) {
        Ok(_) => Ok(ApiResponse::new(
            200,
            json!({ "message": "Email sent successfully" }),
        )),
        Err(e) => Err(ApiResponse::new(
            500,
            json!({ "message": format!("SMTP error: {}", e) }),
        )),
    }
}

async fn send_sms(phone_number: String, message: String) -> Result<Response, ApiResponse> {
    let vaspro_api_key = (*constants::VASPRO_API_KEY).clone();
    let vaspro_shortcode = (*constants::VASPROT_SHORTCODE).clone();

    log::debug!("Preparing to send SMS to: {}", phone_number);

    let client = Client::new();

    let payload = json!({
        "apiKey": vaspro_api_key,
        "shortCode": vaspro_shortcode,
        "message": message,
        "recipient": phone_number,
        "callbackURL": "",
        "enqueue": 0
    });

    log::debug!("Sending SMS request to Vaspro API");

    let res = match client
        .post("https://api.vaspro.co.ke/v3/BulkSMS/api/create")
        .header("Content-Type", "application/json")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(response) => {
            log::info!(
                "SMS API request sent successfully, status: {}",
                response.status()
            );
            response
        }
        Err(err) => {
            let error_msg = format!("SMS API request failed: {}", err);
            log::error!("{}", error_msg);
            return Err(ApiResponse::new(
                500,
                json!({
                    "message": error_msg
                }),
            ));
        }
    };

    Ok(res)
}