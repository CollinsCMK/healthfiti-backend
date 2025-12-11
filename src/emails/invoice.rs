use actix_web::HttpRequest;
use chrono::{Datelike, Utc};
use rust_decimal::Decimal;
use serde_json::json;

use crate::{
    emails::config::email_configs,
    utils::{
        api_response::ApiResponse, app_state::AppState, html_to_image::generate_receipt_png,
        message_queue::MessageType,
    },
};

pub async fn send_invoice_email(
    to: String,
    username: &str,
    invoice_number: &str,
    invoice_date: &str,
    due_date: &str,
    items: Vec<(&str, u32, Decimal)>, // (description, quantity, price)
    total_amount: &Decimal,
    app_state: &AppState,
    req: &HttpRequest,
    domain: Option<String>,
    subject: Option<String>,
    html: Option<String>,
) -> Result<String, ApiResponse> {
    let (
        email_logo,
        email_privacy,
        app_name,
        primary_color,
        accent_color,
        text_color,
        footer_text_color,
    ) = email_configs(app_state, domain).await?;

    let year = Utc::now().year();

    let html = html.unwrap_or_else(|| {
        let mut items_html = String::new();
        for (desc, qty, price) in &items {
            let qty = Decimal::from(*qty);
            let line_total = qty * *price;
            items_html.push_str(&format!(
                r#"<tr>
                    <td style="padding: 8px; border: 1px solid #ddd;">{}</td>
                    <td style="padding: 8px; border: 1px solid #ddd; text-align:center;">{}</td>
                    <td style="padding: 8px; border: 1px solid #ddd; text-align:right;">${:.2}</td>
                    <td style="padding: 8px; border: 1px solid #ddd; text-align:right;">${:.2}</td>
                </tr>"#,
                desc, qty, price, line_total
            ));
        }

        format!(
            r#"
        <!doctype html>
        <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
            </head>
            <body style="padding:0; margin:0; box-sizing:border-box; font-family: Arial, sans-serif; background-color:#F7F6F3;">
                <div style="padding:4px; min-height:100vh;">
                    <div style="background-color:white; border-radius:10px; box-shadow:0 4px 6px rgba(0,0,0,0.1); max-width:700px; margin:0 auto;">
                        <header style="display:flex; align-items:center; justify-content:center; background-color:{primary_color}; width:100%; padding:20px 0; border-radius:10px 10px 0 0;">
                            <img src="{}" alt="logo" style="width:200px; height:auto; filter:drop-shadow(2px 2px 4px rgba(0,0,0,0.2));">
                        </header>

                        <div style="padding:30px 20px;">
                            <h1 style="color:{accent_color}; text-align:center; margin-bottom:30px; font-size:28px;">Invoice #{}</h1>
                            <p style="color:{text_color}; font-size:16px; margin-bottom:15px;">Hello {},</p>
                            <p style="color:{text_color}; font-size:16px; margin-bottom:25px;">
                                Thank you for your business. Below is a summary of your invoice.
                            </p>

                            <table style="width:100%; border-collapse:collapse; margin-bottom:25px;">
                                <tr>
                                    <th style="padding:8px; border:1px solid #ddd; text-align:left;">Description</th>
                                    <th style="padding:8px; border:1px solid #ddd; text-align:center;">Qty</th>
                                    <th style="padding:8px; border:1px solid #ddd; text-align:right;">Price</th>
                                    <th style="padding:8px; border:1px solid #ddd; text-align:right;">Total</th>
                                </tr>
                                {}
                                <tr>
                                    <td colspan="3" style="padding:8px; border:1px solid #ddd; text-align:right; font-weight:bold;">Total</td>
                                    <td style="padding:8px; border:1px solid #ddd; text-align:right; font-weight:bold;">KES: {:.2}</td>
                                </tr>
                            </table>

                            <p style="color:{text_color}; font-size:16px;">
                                Invoice Date: {}<br>
                                Due Date: {}
                            </p>

                            <div style="text-align:center; margin:30px 0;">
                                <a href="/" style="display:inline-block; padding:12px 25px; background-color:{accent_color}; color:white; font-weight:bold; text-decoration:none; border-radius:5px;">View Invoice</a>
                            </div>
                        </div>
                    </div>

                    <div style="flex-grow:1"></div>

                    <footer style="text-align:center; font-size:14px; color:{footer_text_color}; margin-top:20px;">
                        &copy; {} {}. All rights reserved.
                        <br>
                        View our <a href="{}" style="cursor:pointer; color:{accent_color}; text-decoration:none; font-weight:bold;">privacy policy</a>.
                    </footer>
                </div>
            </body>
        </html>
        "#,
            email_logo,
            invoice_number,
            username,
            items_html,
            total_amount,
            invoice_date,
            due_date,
            year,
            app_name,
            email_privacy
        )
    });

    app_state
        .message_queue
        .send_message(MessageType::Email {
            to,
            subject: subject.unwrap_or_else(|| format!("Invoice #{invoice_number}")),
            html: html.clone(),
        })
        .await
        .map_err(|e| ApiResponse::new(500, json!({ "message": e })))?;

    let png_html = {
        if let Some(pos) = html.find(r#"<footer"#) {
            &html[..pos]
        } else {
            &html
        }
    };

    let invoice = generate_receipt_png(&png_html, req, app_state)
        .await
        .map_err(|e| {
            log::error!("Failed to generate invoice PNG: {}", e);
            ApiResponse::new(500, json!({ "message": "Failed to generate invoice PNG" }))
        })?;

    Ok(invoice)
}
