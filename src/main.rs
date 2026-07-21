mod aircraft;
mod airport;
mod generator;
mod global;

use dialoguer::{Select, theme::ColorfulTheme};
use generator::SessionType;

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

    Ok(())
}
