use core::fmt;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::fs;

use crate::airport::Airport;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Adc,
    App,
    Ctr,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Adc => "ADC",
            Self::App => "APP",
            Self::Ctr => "CTR",
        };
        write!(f, "{text}")
    }
}

static FILE_ERROR: &str = "File I/O error";
static INPUT_ERROR: &str = "Input error";

pub fn get_airport_configs() -> Vec<String> {
    let airport_config_folder = "data/airports";

    let airport_configs: Vec<String> = fs::read_dir(airport_config_folder)
        .expect(FILE_ERROR)
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter_map(|path| path.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();

    airport_configs
}

pub fn airport_from_json(path: &str) -> Airport {
    let json = fs::read_to_string(path).expect(FILE_ERROR);

    serde_json::from_str(&json).expect(FILE_ERROR)
}

pub struct AppConfig {
    airport: Airport,
    dep_rate: u8,
    arr_rate: u8,
    ramp_time: Option<u8>,
    name: String,
}

pub fn write_output(
    output: String,
    scenario_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = format!("{scenario_name}.txt");
    fs::write(file_name, output)?;
    Ok(())
}

pub fn app_wizard() -> AppConfig {
    let airport_configs = get_airport_configs();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select airport config")
        .default(0)
        .items(&airport_configs)
        .interact()
        .expect(INPUT_ERROR);

    let airport = airport_from_json(&format!("data/airports/{}", airport_configs[selection]));

    let dep_rate = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Select departure rate (aircraft per hour)")
        .interact()
        .expect(INPUT_ERROR);

    let arr_rate = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Select arrival rate (aircraft per hour)")
        .interact()
        .expect(INPUT_ERROR);

    let ramp_time: Option<u8> = if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to specify a ramp up time?")
        .interact()
        .expect(INPUT_ERROR)
    {
        let ramp = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Specify ramp time in minutes")
            .interact()
            .expect(INPUT_ERROR);
        Some(ramp)
    } else {
        None
    };

    let name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name this scenario")
        .interact()
        .expect(INPUT_ERROR);

    AppConfig {
        airport,
        dep_rate,
        arr_rate,
        ramp_time,
        name,
    }
}

pub fn generate_app() {
    let config = app_wizard();
    let airport = &config.airport;

    let output: String = format!(
        "PSEUDOPILOT:ALL\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
        airport.format_elevation(),
        airport.format_runways(),
        airport.format_holds(),
        airport.format_custom_routes(),
        airport.format_controllers()
    );

    match write_output(output, config.name) {
        Ok(()) => (),
        Err(e) => println!("We couldn't write the file\n\n{e}"),
    }
}
