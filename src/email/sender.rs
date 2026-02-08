use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::error;

use crate::config::{SmtpConfig, SmtpTls};

#[derive(Clone)]
pub struct Mailer {
  transport: AsyncSmtpTransport<Tokio1Executor>,
  from: String,
}

impl Mailer {
  pub fn new(config: &SmtpConfig) -> Result<Self, String> {
    let builder = match config.tls {
      SmtpTls::None => {
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host).port(config.port)
      }
      SmtpTls::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
        .map_err(|e| format!("SMTP STARTTLS error: {e}"))?
        .port(config.port),
      SmtpTls::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
        .map_err(|e| format!("SMTP TLS error: {e}"))?
        .port(config.port),
    };

    let builder = if let (Some(user), Some(pass)) = (&config.username, &config.password) {
      builder.credentials(Credentials::new(user.clone(), pass.clone()))
    } else {
      builder
    };

    Ok(Self {
      transport: builder.build(),
      from: config.from.clone(),
    })
  }

  pub async fn send(&self, to: &str, subject: &str, html: &str) -> Result<(), String> {
    let message = Message::builder()
      .from(
        self
          .from
          .parse()
          .map_err(|e| format!("Invalid from address: {e}"))?,
      )
      .to(to.parse().map_err(|e| format!("Invalid to address: {e}"))?)
      .subject(subject)
      .header(ContentType::TEXT_HTML)
      .body(html.to_string())
      .map_err(|e| format!("Failed to build email: {e}"))?;

    self.transport.send(message).await.map_err(|e| {
      error!("Failed to send email to {}: {}", to, e);
      format!("Failed to send email: {e}")
    })?;

    Ok(())
  }
}
