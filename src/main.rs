mod aircraft;
mod airport;
mod generator;
mod global;
mod route_parser;
mod update_check;

use std::io::{self, Write};

use dialoguer::{Select, theme::ColorfulTheme};
use generator::SessionType;

use crate::generator::{generate_adc, generate_app};
use crate::update_check::check_for_updates;

#[cfg(test)]
mod data_tests;

fn press_enter_to_exit() {
    print!("Press Enter to exit...");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

pub fn config_error(message: &str) -> ! {
    eprintln!("\nConfig error: {message}");
    press_enter_to_exit();
    std::process::exit(1)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    check_for_updates();

    let session_types = &[SessionType::Adc, SessionType::App];

    let scenario = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select scenario type")
        .default(1)
        .items(&session_types[..])
        .interact()?;

    match session_types[scenario] {
        SessionType::Adc => generate_adc(),
        SessionType::App => generate_app(),
    }

    press_enter_to_exit();

    Ok(())
}
