use core::fmt;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use rand::{
    rng,
    seq::{IndexedRandom, SliceRandom},
};
use std::fs;

use crate::{
    aircraft::{
        Aircraft,
        FlightRule::I,
        FlightType::{Arrival, Departure},
        assign_squawks,
    },
    airport::{Airport, ArrivalRoute, DepartureRoute, PositionRoute, Runway},
    generator::SessionType::{Adc, App, Ctr},
    route_parser::{
        RouteType::{Filed, Flown},
        route_parser,
    },
};

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
    selected_runway: usize,
    dep_interval: u8,
    arr_interval: u8,
    duration: u8,
    ramp_time: Option<u8>,
    name: String,
    initial_pseudo_pilot: String,
}

impl AppConfig {
    pub fn runway(&self) -> &Runway {
        &self.airport.runways[self.selected_runway]
    }
}

pub fn write_output(
    output: String,
    scenario_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = format!("{scenario_name}.txt");
    fs::write(&file_name, output)?;
    println!("\nFile written to {}", &file_name);
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

    let runways = &airport.runways;

    let runway_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select runway")
        .default(0)
        .items(runways)
        .interact()
        .expect(INPUT_ERROR);

    let dep_interval = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter departure interval (aircraft every N minutes)")
        .validate_with(|val: &u8| -> Result<(), &str> {
            if *val >= 2 {
                Ok(())
            } else {
                Err("Departure interval must be at least 2 per minute")
            }
        })
        .interact()
        .expect(INPUT_ERROR);

    let arr_interval = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter arrival interval (aircraft every N minutes)")
        .interact()
        .expect(INPUT_ERROR);

    let duration = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter session duration in minutes")
        .interact()
        .expect(INPUT_ERROR);

    let ramp_time: Option<u8> = if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to specify a ramp up time?")
        .interact()
        .expect(INPUT_ERROR)
    {
        let ramp = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter ramp time in minutes")
            .interact()
            .expect(INPUT_ERROR);
        Some(ramp)
    } else {
        None
    };

    let default_pseudo_pilot: String = format!("{}_M_APP", airport.icao);

    let initial_pseudo_pilot: String = if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Would you like to override the default pseudo pilot? {default_pseudo_pilot}"
        ))
        .interact()
        .expect(INPUT_ERROR)
    {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter pseudo pilot callsign")
            .validate_with({
                let mut force = None;
                move |input: &String| -> Result<(), &str> {
                    if input.contains('_') || (force.as_ref() == Some(input)) {
                        Ok(())
                    } else {
                        force = Some(input.clone());
                        Err("That doesn't look like a mentor callsign. Enter again to force procede")
                    }
                }
            })
            .interact()
            .expect(INPUT_ERROR)
    } else {
        default_pseudo_pilot
    };

    let name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name this scenario")
        .interact()
        .expect(INPUT_ERROR);

    AppConfig {
        airport,
        selected_runway: runway_index,
        dep_interval,
        arr_interval,
        duration,
        ramp_time,
        name,
        initial_pseudo_pilot,
    }
}

#[derive(Debug)]
pub struct StagedAircraft {
    pub callsign: String,
    pub outstation: String,
    pub aircraft_type: String,
    pub filed_route: String,
    pub flown_route: String,
    pub spawn_coords: Option<String>,
    pub spawn_alt: Option<u16>,
}

pub fn spawn_timings(session_duration: f32, target_interval: f32, ramp: Option<u8>) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();

    let ramp_time = match ramp {
        Some(s) => s,
        None => 0,
    };
    let mut time: f32 = 0.0;

    while time < session_duration {
        if time < ramp_time as f32 {
            out.push(time.round() as u16);
            let gap = target_interval as f32 * (2.0 - time / ramp_time as f32);
            time += gap
        } else {
            out.push(time.round() as u16);
            time += target_interval
        }
    }
    out
}

pub fn stage_app_departures(
    departure_routes: &[DepartureRoute],
    config: &AppConfig,
) -> Vec<StagedAircraft> {
    let mut out = Vec::new();

    for route in departure_routes {
        for c in &route.callsigns {
            let callsign = c.to_string();
            let outstation = route.dest.to_string();
            let aircraft_type = &route
                .types
                .choose(&mut rng())
                .expect("No aircraft type provided for one or more routes");
            let filed_route = route_parser(
                &route.filed_route,
                &config.airport.standard_routes,
                &config.runway().designator,
                &Filed,
            );
            let flown_route = route_parser(
                &route.flown_route,
                &config.airport.standard_routes,
                &config.runway().designator,
                &Flown,
            );

            out.push(StagedAircraft {
                callsign,
                outstation,
                aircraft_type: aircraft_type.to_string(),
                filed_route,
                flown_route,
                spawn_coords: None,
                spawn_alt: None,
            });
        }
    }
    out
}

pub fn stage_app_arrivals(
    arrival_routes: &[ArrivalRoute],
    config: &AppConfig,
) -> Vec<StagedAircraft> {
    let mut out = Vec::new();

    for route in arrival_routes {
        if let Some(pos) = &route.app_route {
            for c in &route.callsigns {
                let callsign = c.to_string();
                let outstation = route.dep.to_string();
                let aircraft_type = &route
                    .types
                    .choose(&mut rng())
                    .expect("No aircraft type provided for one or more routes");
                let filed_route = route_parser(
                    &route.filed_route,
                    &config.airport.standard_routes,
                    &config.runway().designator,
                    &Filed,
                );
                let flown_route = route_parser(
                    &pos.flown_route,
                    &config.airport.standard_routes,
                    &config.runway().designator,
                    &Flown,
                );
                let spawn_coords = Some(pos.spawn_coords.clone());
                let spawn_alt = Some(pos.spawn_alt);

                out.push(StagedAircraft {
                    callsign,
                    outstation,
                    aircraft_type: aircraft_type.to_string(),
                    filed_route,
                    flown_route,
                    spawn_coords,
                    spawn_alt,
                });
            }
        }
    }
    out
}

pub fn app_arrivals(config: &AppConfig) -> Vec<Aircraft> {
    let mut out: Vec<Aircraft> = Vec::new();

    let times = spawn_timings(
        config.duration as f32,
        config.arr_interval as f32,
        config.ramp_time,
    );
    let mut staged_aircraft = stage_app_arrivals(&config.airport.arrival_routes, config);

    let mut rng = rng();
    staged_aircraft.shuffle(&mut rng);

    for (t, a) in times.iter().zip(staged_aircraft) {
        let aircraft = Aircraft {
            flight_type: Arrival,
            flight_rule: I,
            callsign: a.callsign,
            aircraft_type: a.aircraft_type,
            squawk: None,
            spawn_coords: a.spawn_coords.expect("arrival staged without spawn coords"),
            spawn_altitude: a.spawn_alt.expect("arrival staged without spawn alt"),
            spawn_hdg: None, //not needed for arrivals
            origin: a.outstation,
            dest: config.airport.icao.clone(),
            filed_route: a.filed_route,
            tas: Some(250), //placeholder
            rfl: None,
            flown_route: a.flown_route,
            start: *t,
            delay: None,
            initial_pseudo_pilot: config.initial_pseudo_pilot.clone(),
        };
        out.push(aircraft);
    }
    out
}

pub fn app_departures(config: &AppConfig) -> Vec<Aircraft> {
    let mut out: Vec<Aircraft> = Vec::new();

    let times = spawn_timings(
        config.duration as f32,
        config.dep_interval as f32,
        config.ramp_time,
    );
    let mut staged_aircraft = stage_app_departures(&config.airport.departure_routes, config);

    let mut rng = rng();
    staged_aircraft.shuffle(&mut rng);

    for (t, a) in times.iter().zip(staged_aircraft) {
        let aircraft = Aircraft {
            flight_type: Departure,
            flight_rule: I,
            callsign: a.callsign,
            aircraft_type: a.aircraft_type,
            squawk: None,
            spawn_coords: config.runway().dep_spawn.clone(),
            spawn_altitude: config.airport.round_elevation(),
            spawn_hdg: None, //not needed for radar departures
            origin: config.airport.icao.clone(),
            dest: a.outstation,
            filed_route: a.filed_route,
            tas: Some(250), //placeholder
            rfl: None,
            flown_route: a.flown_route,
            start: *t,
            delay: None,
            initial_pseudo_pilot: config.initial_pseudo_pilot.clone(),
        };
        out.push(aircraft);
    }
    out
}

pub fn generate_app() {
    let config = app_wizard();
    let airport = &config.airport;

    let mut aircraft = app_departures(&config);
    aircraft.extend(app_arrivals(&config));
    assign_squawks(&mut aircraft);

    let ifr_traffic = aircraft
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join("\n\n");

    let output: String = format!(
        "PSEUDOPILOT:ALL\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
        airport.format_elevation(),
        airport.format_runways(),
        airport.format_holds(),
        airport.format_custom_routes(),
        airport.format_controllers(),
        ifr_traffic
    );

    match write_output(output, config.name) {
        Ok(()) => (),
        Err(e) => println!("Couldn't write the file\n\n{e}"),
    }
}
