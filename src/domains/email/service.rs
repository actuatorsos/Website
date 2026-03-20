//! Email Service — خدمة إرسال البريد الإلكتروني عبر SMTP
//!
//! تستخدم مكتبة lettre لإرسال الإيميلات

use super::models::*;
use super::repository;
use crate::db::AppState;

/// Send an email to a list of recipients using the active SMTP config
pub async fn send_email(
    state: &AppState,
    request: &SendEmailRequest,
) -> Result<SendEmailResponse, String> {
    // 1. Get active SMTP config
    let config = repository::get_active_config(state)
        .await
        .map_err(|e| format!("Failed to load email config: {}", e))?
        .ok_or_else(|| {
            "No active email configuration found. Please configure SMTP settings first.".to_string()
        })?;

    // 2. Resolve body (template or direct)
    let final_body = if let Some(ref tmpl_id) = request.template_id {
        match repository::get_template(state, tmpl_id).await {
            Ok(template) => {
                let mut body = template.body.clone();
                if let Some(ref vars) = request.variables {
                    for (key, value) in vars {
                        body = body.replace(&format!("{{{{{}}}}}", key), value);
                    }
                }
                body
            }
            Err(_) => request.body.clone(),
        }
    } else {
        request.body.clone()
    };

    // 3. Build SMTP transport
    let transport = build_transport(&config)?;

    // 4. Send to each recipient
    let mut results = Vec::new();
    let mut sent_count = 0usize;
    let mut failed_count = 0usize;

    for recipient in &request.recipients {
        let result = send_single_email(
            &transport,
            &config.from_email,
            &config.from_name,
            recipient,
            &request.subject,
            &final_body,
        )
        .await;

        let (status, error) = match result {
            Ok(()) => {
                sent_count += 1;
                ("sent".to_string(), None)
            }
            Err(e) => {
                failed_count += 1;
                ("failed".to_string(), Some(e))
            }
        };

        // Log the send attempt
        let _ = repository::create_log(
            state,
            recipient,
            &request.subject,
            Some(&final_body),
            &status,
            error.as_deref(),
        )
        .await;

        results.push(SendResult {
            recipient: recipient.clone(),
            status,
            error,
        });
    }

    Ok(SendEmailResponse {
        sent: sent_count,
        failed: failed_count,
        results,
    })
}

/// Build the SMTP transport from config
fn build_transport(
    config: &EmailConfig,
) -> Result<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>, String> {
    use lettre::transport::smtp::authentication::Credentials;

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let builder = if config.use_tls {
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&config.host)
            .map_err(|e| format!("SMTP relay error: {}", e))?
            .port(config.port as u16)
            .credentials(creds)
    } else {
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(&config.host)
            .port(config.port as u16)
            .credentials(creds)
    };

    Ok(builder.build())
}

/// Send a single email
async fn send_single_email(
    transport: &lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
    from_email: &str,
    from_name: &str,
    to_email: &str,
    subject: &str,
    body_html: &str,
) -> Result<(), String> {
    use lettre::message::header::ContentType;
    use lettre::{AsyncTransport, Message};

    let from_addr = format!("{} <{}>", from_name, from_email);

    let email = Message::builder()
        .from(
            from_addr
                .parse()
                .map_err(|e| format!("Invalid from address: {}", e))?,
        )
        .to(to_email
            .parse()
            .map_err(|e| format!("Invalid recipient address '{}': {}", to_email, e))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body_html.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    transport
        .send(email)
        .await
        .map_err(|e| format!("SMTP send error: {}", e))?;

    Ok(())
}
