use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::{error::Error, time::Duration};
use thirtyfour::{FirefoxCapabilities, prelude::*};
use tokio::process::Command;
use tokio::signal;
use um_scraper::config;
use um_scraper::config::Config;
use um_scraper::grade::Grade;
use um_scraper::mail::MailManager;

async fn ent_login(driver: &WebDriver, config: &config::Config) -> Result<(), Box<dyn Error>> {
    driver.goto("https://ent.umontpellier.fr").await?;

    let username_field = driver
        .query(By::Id("username"))
        .wait(config.sleep_time, Duration::from_secs(1))
        .first()
        .await?;
    let password_field = driver
        .query(By::Id("password"))
        .wait(config.sleep_time, Duration::from_secs(1))
        .first()
        .await?;
    username_field.send_keys(&config.ent_login_email).await?;
    password_field.send_keys(&config.ent_password).await?;

    let login_button = driver
        .query(By::Css("[type=submit]"))
        .wait(config.sleep_time, Duration::from_secs(1))
        .first()
        .await?;
    login_button.click().await?;

    Ok(())
}

async fn fetch_grades(
    driver: &WebDriver,
    config: &config::Config,
) -> Result<Vec<Grade>, Box<dyn Error>> {
    driver
        .goto("https://app.umontpellier.fr/mdw/#!notesView")
        .await?;

    let semester_table = driver
        .query(By::ClassName("v-table-body-noselection"))
        .wait(config.sleep_time, Duration::from_secs(1))
        .all_from_selector()
        .await?[1]
        .clone();

    let last_semester_link = semester_table
        .query(By::ClassName("v-table-cell-content"))
        .wait(config.sleep_time, Duration::from_secs(1))
        .all_from_selector()
        .await?[1]
        .clone();
    last_semester_link.click().await?;

    tokio::time::sleep(Duration::from_secs(10)).await;

    let grades_table = driver
        .query(By::ClassName("v-table-table"))
        .wait(config.sleep_time, Duration::from_secs(1))
        .all_from_selector()
        .await?
        .last()
        .unwrap()
        .clone();

    let grades_row = grades_table
        .query(By::Tag("tr"))
        .wait(config.sleep_time, Duration::from_secs(1))
        .all_from_selector()
        .await?;

    let mut grades = Vec::new();
    for grade_row in grades_row.into_iter().skip(2) {
        if let Ok(grade) = Grade::from_row(grade_row).await {
            grades.push(grade);
        }
    }

    Ok(grades)
}

struct RunArgs<'a> {
    capabilities: FirefoxCapabilities,
    last_grades: &'a mut Vec<Grade>,
    last_grades_set: &'a mut HashSet<Grade>,
}

async fn run<'a>(args: &mut RunArgs<'a>) -> Result<(), Box<dyn Error>> {
    log::info!("Refreshing config...");
    let config = Config::load();
    let mail_manager = MailManager::new(&config);

    let driver = WebDriver::new(
        &format!("http://localhost:{}", config.geckodriver_port),
        args.capabilities.clone(),
    )
    .await?;

    log::info!("Logging to ENT...");
    ent_login(&driver, &config).await?;

    log::info!("Fetching grades...");
    let fetched_grades = fetch_grades(&driver, &config).await;

    log::info!("Killing geckodriver...");
    driver.quit().await?;
    let fetched_grades = fetched_grades?;

    let fetched_grades_set = fetched_grades.iter().cloned().collect::<HashSet<_>>();
    if fetched_grades_set.eq(args.last_grades_set) {
        log::info!("Data didn't changed...");
    } else {
        log::info!("Data changed !");

        log::info!("Sending mail...");
        mail_manager
            .send_objects(
                &config.to_emails,
                &fetched_grades
                    .iter()
                    .filter(|grade| !args.last_grades_set.contains(grade))
                    .collect(),
            )
            .expect("Failed to send mails");

        log::info!("Writing new data to file...");
        fs::write(config.data_file, serde_yaml::to_string(&fetched_grades)?)?;
    }

    *args.last_grades = fetched_grades.clone();
    *args.last_grades_set = args.last_grades.iter().cloned().collect::<HashSet<_>>();

    Ok(())
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

    let config = Config::load();
    log::info!("Reading file data...");
    let mut last_grades: Vec<Grade> = std::fs::File::open(&config.data_file)
        .map(serde_yaml::from_reader)
        .unwrap_or(Ok(Vec::new()))
        .unwrap_or_default();
    let mut last_grades_set = last_grades.iter().cloned().collect::<HashSet<_>>();
    log::info!("last_grades: {last_grades:?}");

    log::info!("Spawning geckodriver...");
    let mut geckodiver = Command::new("geckodriver")
        .arg("--port")
        .arg(config.geckodriver_port.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let _ = tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).unwrap();

        tokio::select! {
            _ = sigterm.recv() => log::info!("Received SIGTERM"),
            _ = sigint.recv() => log::info!("Received SIGINT"),
        }

        geckodiver.kill().await.unwrap();
        std::process::exit(0);
    });

    let mut capabilities = DesiredCapabilities::firefox();
    capabilities.set_headless().unwrap();

    let mut args = RunArgs {
        capabilities,
        last_grades: &mut last_grades,
        last_grades_set: &mut last_grades_set,
    };

    tokio::time::sleep(Duration::from_secs(1)).await;
    loop {
        match run(&mut args).await {
            Ok(()) => (),
            Err(e) => log::error!("Runtime error: {e}"),
        }
        log::info!(
            "Waiting {}...",
            humantime::format_duration(config.sleep_time).to_string()
        );
        tokio::time::sleep(config.sleep_time).await;
    }
}
