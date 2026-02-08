use serde::Serialize;
use serde_derive::*;
use std::{path::PathBuf, time::Duration};

pub const CONFIG_PATH: &str = "./umscraper.yaml";

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ent_login_email: String,
    pub ent_password: String,
    pub gmail_login_email: String,
    pub gmail_login_password: String,
    pub gmail_from_email: Option<String>,
    pub to_emails: Vec<String>,
    pub data_file: PathBuf,
    #[serde(with = "humantime_serde")]
    pub sleep_time: Duration,
    pub geckodriver_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ent_login_email: "[...]@etu.umontpellier.fr".to_string(),
            ent_password: "...".to_string(),
            gmail_login_email: "[...]@gmail.[...]".to_string(),
            gmail_login_password: "app password".to_string(),
            gmail_from_email: Some("different mail (alias) or remove this part".to_string()),
            to_emails: vec![
                "mail1@gmail.com".to_string(),
                "mail2@hotmail.com".to_string(),
            ],
            data_file: PathBuf::from("./grades.yaml"),
            sleep_time: durations::MINUTE * 10,
            geckodriver_port: 4444,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = PathBuf::from(CONFIG_PATH);
        if !config_path.exists() {
            let default_config = Self::default();
            std::fs::write(
                &config_path,
                serde_yaml::to_string(&default_config).unwrap(),
            )
            .unwrap();
            return default_config;
        }
        let file = std::fs::File::open(&config_path).unwrap();
        let config: Config = serde_yaml::from_reader(file).unwrap();

        if let Err(error) = config.gmail_login_email.parse::<lettre::Address>() {
            panic!(
                "wrong login_email {} : {:#?}",
                config.gmail_login_email, error
            );
        }

        if let Some(Err(error)) = config
            .gmail_from_email
            .clone()
            .map(|email| email.parse::<lettre::Address>())
        {
            panic!(
                "wrong from_email {:?} : {:#?}",
                config.gmail_from_email, error
            );
        }

        let wrong_to_emails = config
            .to_emails
            .iter()
            .filter_map(|email| match email.parse::<lettre::Address>() {
                Ok(_) => None,
                Err(error) => Some((email, error)),
            })
            .collect::<Vec<_>>();
        if !wrong_to_emails.is_empty() {
            panic!("wrong to_emails : {:#?}", wrong_to_emails);
        }

        config
    }
}
