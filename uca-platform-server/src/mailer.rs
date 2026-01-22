//! 邮件发送工具。

use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::config::MailConfig;
use crate::error::AppError;

pub async fn send_mail(config: &MailConfig, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
    let from = if let Some(name) = &config.from_name {
        Mailbox::new(Some(name.to_string()), config.from_address.parse().map_err(|_| AppError::config("invalid mail from address"))?)
    } else {
        Mailbox::new(None, config.from_address.parse().map_err(|_| AppError::config("invalid mail from address"))?)
    };
    let to_mailbox = Mailbox::new(None, to.parse().map_err(|_| AppError::validation("invalid email"))?);
    let message = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject(subject)
        .body(body.to_string())
        .map_err(|_| AppError::internal("failed to build email"))?;

    let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());
    let mailer = if config.use_tls {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
            .map_err(|_| AppError::config("invalid smtp host"))?
            .port(config.smtp_port)
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
            .map_err(|_| AppError::config("invalid smtp host"))?
            .port(config.smtp_port)
            .credentials(creds)
            .build()
    };

    mailer
        .send(message)
        .await
        .map_err(|_| AppError::internal("failed to send email"))?;
    Ok(())
}
