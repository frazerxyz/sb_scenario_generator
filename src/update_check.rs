use std::{thread, time::Duration};

use dialoguer::{Confirm, theme::ColorfulTheme};
use serde::Deserialize;
use spinners::{Spinner, Spinners};

static RELEASES_URL: &str =
    "https://api.github.com/repos/frazerxyz/sb_scenario_generator/releases/latest";

const UPDATE_ATTEMPTS: u8 = 5;
const RETRY_DELAY: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ERROR_LENGTH: usize = 120;

// remove URL from reqwest error
fn tidy_error(error: &str) -> &str {
    match error.find(" for url (") {
        Some(i) => &error[..i],
        None => error,
    }
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

// Ok(None) if no release present. Not a fetching error so no retry
fn fetch_latest_release() -> Result<Option<Release>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    let response = client.get(RELEASES_URL).send()?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let body = response.error_for_status()?.text()?;

    Ok(Some(serde_json::from_str(&body)?))
}

fn report_release(release: &Release) {
    let current = env!("CARGO_PKG_VERSION");
    let latest = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);

    if latest == current {
        return;
    }

    println!("Update available");
    println!("  Your version:   {current}");
    println!("  Latest version: {latest}");

    if let Err(e) = opener::open_browser(&release.html_url) {
        println!("Couldn't open the release page ({e})");
        println!("{}", release.html_url);
    }
}

pub fn check_for_updates() {
    let mut last_error = String::new();

    for attempt in 1..=UPDATE_ATTEMPTS {
        let mut spinner = Spinner::new(Spinners::Dots, "Checking for updates".into());

        if attempt > 1 {
            thread::sleep(RETRY_DELAY);
        }

        match fetch_latest_release() {
            Ok(Some(release)) => {
                spinner.stop_with_message(String::new());
                report_release(&release);
                return;
            }
            Ok(None) => {
                spinner.stop_with_message("No releases published yet".into());
                return;
            }
            Err(e) => {
                last_error = e.to_string();
                spinner.stop_with_message(format!(
                    "Couldn't reach GitHub (attempt {attempt} of {UPDATE_ATTEMPTS})"
                ));
            }
        }

        // offer to cancel retries
        if attempt == 1
            && !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Keep trying to check for updates?")
                .default(true)
                .interact()
                .unwrap_or(false)
        {
            break;
        }
    }

    println!("Failed to check for updates");

    let message = tidy_error(&last_error);
    if !message.is_empty() && message.len() <= MAX_ERROR_LENGTH {
        println!("{message}");
    }
}

pub fn fetch_data_file(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut spinner = Spinner::new(
        Spinners::Dots,
        "Checking for airport data file updates".into(),
    );

    let res = reqwest::blocking::get(url)?.error_for_status()?.text()?;

    spinner.stop_with_message(String::new());

    Ok(res)
}
