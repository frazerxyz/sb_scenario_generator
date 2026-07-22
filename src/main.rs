mod aircraft;
mod airport;
mod generator;
mod global;
mod route_parser;

use std::io::{self, Write};

use dialoguer::{Select, theme::ColorfulTheme};
use generator::SessionType;

use crate::generator::generate_app;

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
    let session_types = &[SessionType::Adc, SessionType::App, SessionType::Ctr];

    let scenario = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select scenario type")
        .default(1)
        .items(&session_types[..])
        .interact()?;

    match session_types[scenario] {
        SessionType::Adc => todo!("Write ADC generator"),
        SessionType::App => generate_app(),
        SessionType::Ctr => todo!("Write CTR generator"),
    }

    press_enter_to_exit();

    Ok(())
}
