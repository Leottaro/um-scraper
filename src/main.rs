use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::{error::Error, time::Duration};
use thirtyfour::prelude::*;
use tokio::process::Command;
use um_scraper::config;
use um_scraper::config::Config;
use um_scraper::mail::MailManager;
use um_scraper::note::Note;

async fn ent_login(driver: &WebDriver, config: &config::Config) -> Result<(), Box<dyn Error>> {
    driver.goto("https://ent.umontpellier.fr").await?;

    let username_field = driver
        .query(By::Id("username"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .first()
        .await?;
    let password_field = driver
        .query(By::Id("password"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .first()
        .await?;
    username_field.send_keys(&config.ent_login_email).await?;
    password_field.send_keys(&config.ent_password).await?;

    let login_button = driver
        .query(By::Css("[type=submit]"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .first()
        .await?;
    login_button.click().await?;
    driver
        .set_implicit_wait_timeout(Duration::from_secs(10))
        .await?;
    Ok(())
}

async fn fetch_notes(driver: &WebDriver) -> Result<Vec<Note>, Box<dyn Error>> {
    driver
        .goto("https://app.umontpellier.fr/mdw/#!notesView")
        .await?;

    let semester_table = driver
        .query(By::ClassName("v-table-body-noselection"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .all_from_selector()
        .await?[1]
        .clone();

    let last_semester_link = semester_table
        .query(By::ClassName("v-table-cell-content"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .all_from_selector()
        .await?[1]
        .clone();
    last_semester_link.click().await?;

    tokio::time::sleep(Duration::from_secs(10)).await;

    let notes_table = driver
        .query(By::ClassName("v-table-table"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .all_from_selector()
        .await?
        .last()
        .unwrap()
        .clone();

    let notes_row = notes_table
        .query(By::Tag("tr"))
        .wait(Duration::from_secs(10), Duration::from_millis(500))
        .all_from_selector()
        .await?;

    let mut notes = Vec::new();
    for note_row in notes_row.into_iter().skip(2) {
        if let Ok(note) = Note::from_row(note_row).await {
            notes.push(note);
        }
    }

    Ok(notes)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_dimmed(true);
            let timestamp = style.value(
                buf.timestamp()
                    .to_string()
                    .replace("-", "/")
                    .replace("T", " ")
                    .replace("Z", ""),
            );

            writeln!(
                buf,
                "[{} {}:{} {} {}] {}",
                timestamp,
                record.file().unwrap_or_default(),
                record.line().unwrap_or_default(),
                record.target(),
                buf.default_styled_level(record.level()),
                record.args()
            )
        })
        .init();

    let mut config = Config::load();
    let mut mail_manager = MailManager::new(&config);

    log::info!("Reading file data...");
    let mut last_notes: Vec<Note> = std::fs::File::open(&config.data_file)
        .map(serde_yaml::from_reader)
        .unwrap_or(Ok(Vec::new()))
        .unwrap_or_default();
    let mut last_notes_set = last_notes.iter().cloned().collect::<HashSet<_>>();
    log::info!("last_notes: {last_notes:?}");

    loop {
        log::info!("Spawning geckodriver...");
        let mut geckodriver = Command::new("geckodriver")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut caps = DesiredCapabilities::firefox();
        caps.set_headless().unwrap();
        let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();

        log::info!("Logging to ENT...");
        ent_login(&driver, &config).await.unwrap();

        log::info!("Fetching notes...");
        let res = fetch_notes(&driver).await;

        log::info!("Killing geckodriver...");
        driver.quit().await.unwrap();
        geckodriver.kill().await.unwrap();
        let fetched_notes = match res {
            Ok(notes) => notes,
            Err(_) => continue,
        };

        let fetched_notes_set = fetched_notes.iter().cloned().collect::<HashSet<_>>();
        if last_notes_set.eq(&fetched_notes_set) {
            log::info!("Data didn't changed...");
        } else {
            log::info!("Data changed !");

            log::info!("Sending mail...");
            mail_manager
                .send_objects(
                    &config.to_emails,
                    &fetched_notes
                        .iter()
                        .filter(|note| !last_notes_set.contains(note))
                        .collect(),
                )
                .expect("Failed to send mails");

            log::info!("Writing new data to file...");
            fs::write(
                config.data_file,
                serde_yaml::to_string(&fetched_notes).unwrap(),
            )
            .unwrap();
        }

        log::info!(
            "Waiting {}...",
            humantime::format_duration(config.sleep_time).to_string()
        );
        last_notes = fetched_notes.clone();
        last_notes_set = last_notes.iter().cloned().collect::<HashSet<_>>();
        tokio::time::sleep(config.sleep_time).await;

        log::info!("Refreshing config...");
        config = Config::load();
        mail_manager = MailManager::new(&config);
    }
}
