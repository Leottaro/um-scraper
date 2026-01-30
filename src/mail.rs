use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::config;
use lettre::{
    Address, Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::{authentication::Credentials, response::Response},
};
use serde::Serialize;

pub struct MailManager {
    mailer: SmtpTransport,
    mailbox: Mailbox,
}

impl MailManager {
    pub fn new(config: &config::Config) -> Self {
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(
                config.gmail_login_email.clone(),
                config.gmail_login_password.clone(),
            ))
            .build();

        let from_email = config
            .gmail_from_email
            .as_ref()
            .unwrap_or(&config.gmail_login_email)
            .clone();
        let mailbox = Mailbox::new(Some("Web scraper".to_string()), from_email.parse().unwrap());

        Self { mailer, mailbox }
    }

    pub fn send_objects<T>(
        &self,
        to_emails: &Vec<String>,
        objects: &Vec<T>,
    ) -> Result<Vec<Response>, ()>
    where
        T: Serialize,
    {
        let to_emails = to_emails
            .iter()
            .map(|mail| mail.parse::<Address>().unwrap())
            .collect::<Vec<_>>();
        let body = serde_yaml::to_string(objects).unwrap();

        log::info!("Sending mails...");
        let mailer = Arc::new(Mutex::new(self.mailer.clone()));
        let mut handles = Vec::new();
        for to_email in to_emails {
            let message = Message::builder()
                .from(self.mailbox.clone())
                .to(Mailbox::new(None, to_email.clone()))
                .subject("HDD changed")
                .header(ContentType::TEXT_PLAIN)
                .body(body.clone())
                .unwrap();

            let mailer = mailer.clone();
            let handle = thread::Builder::new()
                .name(format!("thread-mail-{to_email}"))
                .spawn(move || mailer.lock().unwrap().send(&message).unwrap())
                .unwrap();

            handles.push(handle);
        }

        let mut responses = Vec::new();
        for handle in handles {
            responses.push(handle.join().unwrap());
        }

        Ok(responses)
    }
}
