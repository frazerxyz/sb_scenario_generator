mod aircraft;
mod airport;
mod generator;
mod global;

use airport::Airport;
use dialoguer::{Select, theme::ColorfulTheme};
use generator::SessionType;
use std::fs;

use crate::generator::generate_app;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session_types = &[SessionType::Adc, SessionType::App, SessionType::Ctr];

    let scenario = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select scenario type")
        .default(1)
        .items(&session_types[..])
        .interact()?;

    match session_types[scenario] {
        SessionType::Adc => println!("ADC not supported yet"),
        SessionType::App => generate_app(),
        SessionType::Ctr => println!("CTR not supported yet"),
    }

    let json = fs::read_to_string("data/airports/EGKK.json")?;

    let _a: Airport = serde_json::from_str(&json)?;

    Ok(())
}
