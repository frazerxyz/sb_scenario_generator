use core::fmt;
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use rand::{
    rng,
    seq::{IndexedRandom, SliceRandom},
};
use std::fs;

use crate::{
    aircraft::{
        Aircraft,
        FlightRule::{I, V},
        FlightType::{Arrival, Departure, Local},
        assign_squawks,
    },
    airport::{
        Airline, Airport, ArrivalRoute, DepartureRoute, LocalVfr, ReferenceCode, Runway,
        VfrFlightPlan,
    },
    press_enter_to_exit,
    route_parser::{
        RouteType::{Filed, Flown},
        route_parser,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Adc,
    App,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Adc => "ADC",
            Self::App => "APP",
        };
        write!(f, "{text}")
    }
}

static FILE_ERROR: &str = "File I/O error";
static INPUT_ERROR: &str = "Input error";

static FILE_MESSAGE: &str =
    "; ----- Made using https://github.com/frazerxyz/sb_scenario_generator -----";

pub fn check_file(file_name: &str) {
    match fs::exists(file_name) {
        Ok(true) => {
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("{} already exists, overwrite?", &file_name))
                .interact()
                .expect(INPUT_ERROR)
            {
                println!("Aborting");
                press_enter_to_exit();
                std::process::exit(0)
            }
        }
        Ok(false) => (),
        Err(_) => {
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "There was an error checking if {} already exists. Write anyway overwrite?",
                    &file_name
                ))
                .interact()
                .expect(INPUT_ERROR)
            {
                println!("Aborting");
                press_enter_to_exit();
                std::process::exit(0)
            }
        }
    }
}

pub fn write_output(
    output: String,
    scenario_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = format!("{scenario_name}.txt");
    check_file(&file_name);
    fs::write(&file_name, output)?;
    println!("\nFile written to {}", &file_name);
    Ok(())
}

fn aircraft_perf() -> String {
    match fs::exists("data/aircraft_perf.txt") {
        Ok(true) => fs::read_to_string("data/aircraft_perf.txt").expect(FILE_ERROR),
        Ok(false) => String::from(""),
        Err(e) => {
            println!("Error checking if aircraft performance file exists {e}");
            String::from("")
        }
    }
}

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
    vfr_traffic: Vec<Aircraft>,
}

impl AppConfig {
    pub fn runway(&self) -> &Runway {
        &self.airport.runways[self.selected_runway]
    }
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
                Err("Departure interval must be at least 2 minutes")
            }
        })
        .interact()
        .expect(INPUT_ERROR);

    let arr_interval = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter arrival interval (aircraft every N minutes)")
        .validate_with(|val: &u8| -> Result<(), &str> {
            if *val >= 1 {
                Ok(())
            } else {
                Err("Arrival interval cannot be 0")
            }
        })
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
                    Err("That doesn't look like a mentor callsign. Enter again to force proceed")
                }
            }
        })
        .interact()
        .expect(INPUT_ERROR)
    } else {
        default_pseudo_pilot
    };

    let vfr_traffic = configure_vfr(&airport, &initial_pseudo_pilot);

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
        vfr_traffic,
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
    pub rfl: u16,
}

pub fn spawn_timings(session_duration: f32, target_interval: f32, ramp: Option<u8>) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();

    let ramp_time = ramp.unwrap_or_default();
    let mut time: f32 = 0.0;

    while time < session_duration {
        if time < ramp_time as f32 {
            out.push(time.round() as u16);
            let gap = target_interval * (2.0 - time / ramp_time as f32);
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
            let rfl = route.rfl;

            out.push(StagedAircraft {
                callsign,
                outstation,
                aircraft_type: aircraft_type.to_string(),
                filed_route,
                flown_route,
                spawn_coords: None,
                spawn_alt: None,
                rfl,
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
                let rfl = route.rfl;

                out.push(StagedAircraft {
                    callsign,
                    outstation,
                    aircraft_type: aircraft_type.to_string(),
                    filed_route,
                    flown_route,
                    spawn_coords,
                    spawn_alt,
                    rfl,
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
            rfl: Some(a.rfl),
            flown_route: a.flown_route,
            start: *t,
            delay: None,
            initial_pseudo_pilot: config.initial_pseudo_pilot.clone(),
            has_flight_plan: true,
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
            rfl: Some(a.rfl),
            flown_route: a.flown_route,
            start: *t,
            delay: None,
            initial_pseudo_pilot: config.initial_pseudo_pilot.clone(),
            has_flight_plan: true,
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
    aircraft.extend(config.vfr_traffic);

    let traffic = aircraft
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join("\n\n");

    let output: String = format!(
        "{}\n\nPSEUDOPILOT:ALL\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
        FILE_MESSAGE,
        airport.format_elevation(),
        airport.format_runways(),
        airport.format_holds(),
        airport.format_custom_routes(),
        airport.format_controllers(),
        traffic,
        aircraft_perf(),
    );

    match write_output(output, config.name) {
        Ok(()) => (),
        Err(e) => println!("Couldn't write the file\n\n{e}"),
    }
}

pub fn local_vfr_wizard(local_aircraft: &[LocalVfr]) -> Vec<LocalVfr> {
    let selected = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select local VFR aircraft (space to select, enter to confirm)")
        .items(local_aircraft)
        .interact()
        .expect(INPUT_ERROR);

    selected
        .into_iter()
        .map(|i| local_aircraft[i].clone())
        .collect()
}

struct StagedGroundVfr {
    callsign: String,
    aircraft_type: String,
    flight_plan: VfrFlightPlan,
}

fn ground_vfr_wizard(default_flight_plan: &VfrFlightPlan) -> StagedGroundVfr {
    let theme = ColorfulTheme::default();

    let callsign: String = Input::with_theme(&theme)
        .with_prompt("Enter callsign")
        .validate_with(|callsign: &String| -> Result<(), &str> {
            if callsign.len() <= 7 {
                if callsign.is_empty() {
                    Err("Please enter a callsign")
                } else {
                    Ok(())
                }
            } else {
                Err("Callsign too long. Max 7 characters")
            }
        })
        .interact_text()
        .expect(INPUT_ERROR);

    let aircraft_type: String = Input::with_theme(&theme)
        .with_prompt("Enter aircraft type code")
        .validate_with(|a: &String| -> Result<(), &str> {
            if a.len() <= 4 {
                if a.is_empty() {
                    Err("Please enter a type code")
                } else {
                    Ok(())
                }
            } else {
                Err("Type code length too long. Max 4 characters")
            }
        })
        .interact_text()
        .expect(INPUT_ERROR);

    let flight_plan = default_flight_plan.clone();

    StagedGroundVfr {
        callsign,
        aircraft_type,
        flight_plan,
    }
}

pub fn configure_vfr(airport: &Airport, initial_pseudo_pilot: &str) -> Vec<Aircraft> {
    let mut vfr_aircraft = Vec::new();

    if let Some(a) = &airport.local_vfr {
        for i in local_vfr_wizard(a) {
            let has_flight_plan: bool = !i.flight_plan.dep.is_empty();
            vfr_aircraft.push(Aircraft {
                flight_type: Local,
                flight_rule: V,
                callsign: i.callsign,
                aircraft_type: i.aircraft_type,
                squawk: Some(0o7000),
                spawn_coords: i.spawn_coords,
                spawn_altitude: i.spawn_alt,
                spawn_hdg: None,
                origin: i.flight_plan.dep,
                dest: i.flight_plan.dest,
                filed_route: i.flight_plan.route,
                tas: Some(120),
                rfl: Some(i.flight_plan.alt),
                flown_route: i.route,
                start: 0,
                delay: None,
                initial_pseudo_pilot: initial_pseudo_pilot.to_string(),
                has_flight_plan,
            });
        }
    }

    if let Some(g) = &airport.ground_vfr {
        let mut staged: Vec<StagedGroundVfr> = Vec::new();

        let max = g.spawn_coords.len();

        while staged.len() < max {
            let ad: String = if staged.is_empty() {
                "any".to_string()
            } else {
                "another".to_string()
            };
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Would you like to configure {ad} ground VFR aircraft?"
                ))
                .interact()
                .expect(INPUT_ERROR)
            {
                staged.push(ground_vfr_wizard(&g.default_flight_plan));
            } else {
                break;
            }
        }

        for (s, coord) in staged.into_iter().zip(g.spawn_coords.iter()) {
            vfr_aircraft.push(Aircraft {
                flight_type: Local,
                flight_rule: V,
                callsign: s.callsign,
                aircraft_type: s.aircraft_type,
                squawk: Some(0o7000),
                spawn_coords: coord.clone(),
                spawn_altitude: airport.elevation as u16,
                spawn_hdg: None,
                origin: s.flight_plan.dep,
                dest: s.flight_plan.dest,
                filed_route: s.flight_plan.route,
                tas: Some(120),
                rfl: Some(s.flight_plan.alt),
                flown_route: "".to_string(),
                start: 0,
                delay: None,
                initial_pseudo_pilot: initial_pseudo_pilot.to_string(),
                has_flight_plan: true,
            });
        }
    }

    vfr_aircraft
}

struct AdcConfig {
    airport: Airport,
    arrival_runway: usize,
    departures: u8,
    arr_interval: u8,
    duration: u8,
    name: String,
    initial_pseudo_pilot: String,
    vfr_traffic: Vec<Aircraft>,
}

impl AdcConfig {
    pub fn runway(&self) -> &Runway {
        &self.airport.runways[self.arrival_runway]
    }
}

fn preferred_terminals<'a>(callsign: &str, airlines: &'a [Airline]) -> &'a [u32] {
    let prefix = callsign.get(..3).unwrap_or_default();

    match airlines.iter().find(|a| a.icao == prefix) {
        Some(airline) => &airline.terminals,
        None => &[],
    }
}

fn allocate_stand(
    stands: &mut Vec<DepartureStand>,
    callsign: &str,
    airlines: &[Airline],
) -> Option<DepartureStand> {
    let preferred = preferred_terminals(callsign, airlines);

    match stands.iter().position(|s| preferred.contains(&s.terminal)) {
        Some(i) => Some(stands.remove(i)),
        None => stands.pop(),
    }
}

pub fn generate_adc() {
    let config = adc_wizard();
    let airport = &config.airport;

    let mut aircraft = adc_arrivals(&config);
    assign_squawks(&mut aircraft);
    aircraft.extend(adc_departures(&config));
    aircraft.extend(config.vfr_traffic);

    let traffic = aircraft
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join("\n\n");

    let output: String = format!(
        "{}\n\nPSEUDOPILOT:ALL\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
        FILE_MESSAGE,
        airport.format_elevation(),
        airport.format_runways(),
        airport.format_holds(),
        airport.format_custom_routes(),
        airport.format_controllers(),
        traffic,
        aircraft_perf(),
    );

    match write_output(output, config.name) {
        Ok(()) => (),
        Err(e) => println!("Couldn't write the file\n\n{e}"),
    }
}

fn stage_adc_arrivals(arrival_routes: &[ArrivalRoute], config: &AdcConfig) -> Vec<StagedAircraft> {
    let mut out = Vec::new();

    for route in arrival_routes {
        if let Some(pos) = &route.adc_route {
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
                let rfl = route.rfl;

                out.push(StagedAircraft {
                    callsign,
                    outstation,
                    aircraft_type: aircraft_type.to_string(),
                    filed_route,
                    flown_route,
                    spawn_coords,
                    spawn_alt,
                    rfl,
                });
            }
        }
    }
    out
}

fn adc_arrivals(config: &AdcConfig) -> Vec<Aircraft> {
    let mut out: Vec<Aircraft> = Vec::new();

    let times = spawn_timings(config.duration as f32, config.arr_interval as f32, None);

    let mut staged_aircraft = stage_adc_arrivals(&config.airport.arrival_routes, config);
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
            rfl: Some(a.rfl),
            flown_route: a.flown_route,
            start: *t,
            delay: None,
            initial_pseudo_pilot: config.initial_pseudo_pilot.clone(),
            has_flight_plan: true,
        };
        out.push(aircraft);
    }
    out
}

fn adc_wizard() -> AdcConfig {
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

    let departures = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter number of IFR departures")
        .interact()
        .expect(INPUT_ERROR);

    let arr_interval = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter arrival interval (aircraft every N minutes)")
        .validate_with(|val: &u8| -> Result<(), &str> {
            if *val >= 1 {
                Ok(())
            } else {
                Err("Arrival interval cannot be 0")
            }
        })
        .interact()
        .expect(INPUT_ERROR);

    let duration = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter session duration in minutes")
        .interact()
        .expect(INPUT_ERROR);

    let default_pseudo_pilot: String = format!("{}_M_TWR", airport.icao);

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
                    Err("That doesn't look like a mentor callsign. Enter again to force proceed")
                }
            }
        })
        .interact()
        .expect(INPUT_ERROR)
    } else {
        default_pseudo_pilot
    };

    let vfr_traffic = configure_vfr(&airport, &initial_pseudo_pilot);

    let name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name this scenario")
        .interact()
        .expect(INPUT_ERROR);

    AdcConfig {
        airport,
        arrival_runway: runway_index,
        departures,
        arr_interval,
        duration,
        name,
        initial_pseudo_pilot,
        vfr_traffic,
    }
}

struct DepartureStand {
    number: u32,
    terminal: u32,
    max_size: ReferenceCode,
    coords: String,
    heading: u16,
}

fn generate_stands(airport: &Airport) -> Vec<DepartureStand> {
    let mut stands: Vec<DepartureStand> = Vec::new();

    for t in &airport.terminals {
        for s in &t.stands {
            stands.push(DepartureStand {
                number: s.id,
                terminal: t.id,
                max_size: s.max_size,
                coords: s.coords.clone(),
                heading: es_heading(s.bearing),
            });
        }
    }

    let mut rng = rng();
    stands.shuffle(&mut rng);
    stands
}

fn es_heading(degrees: f32) -> u16 {
    (((degrees * 2.88 + 0.5) as u32) << 2) as u16
}

fn adc_departures(config: &AdcConfig) -> Vec<Aircraft> {
    let mut out: Vec<Aircraft> = Vec::new();

    let mut staged_aircraft = stage_adc_departures(&config.airport.departure_routes, config);

    let mut rng = rng();
    staged_aircraft.shuffle(&mut rng);
    staged_aircraft.truncate(config.departures as usize);

    let mut stands = generate_stands(&config.airport);

    for a in staged_aircraft {
        let stand = match allocate_stand(&mut stands, &a.callsign, &config.airport.airlines) {
            Some(s) => s,
            None => continue,
        };

        let aircraft = Aircraft {
            flight_type: Departure,
            flight_rule: I,
            callsign: a.callsign,
            aircraft_type: a.aircraft_type,
            squawk: Some(0o2000),
            spawn_coords: stand.coords,
            spawn_altitude: config.airport.elevation as u16,
            spawn_hdg: Some(stand.heading),
            origin: config.airport.icao.clone(),
            dest: a.outstation,
            filed_route: a.filed_route,
            tas: Some(250), //placeholder
            rfl: Some(a.rfl),
            flown_route: a.flown_route,
            start: 0,
            delay: None,
            initial_pseudo_pilot: config.initial_pseudo_pilot.clone(),
            has_flight_plan: true,
        };
        out.push(aircraft);
    }
    out
}

fn stage_adc_departures(
    departure_routes: &[DepartureRoute],
    config: &AdcConfig,
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
            let rfl = route.rfl;

            out.push(StagedAircraft {
                callsign,
                outstation,
                aircraft_type: aircraft_type.to_string(),
                filed_route,
                flown_route,
                spawn_coords: None,
                spawn_alt: None,
                rfl,
            });
        }
    }
    out
}

#[test]
fn heading_conversion() {
    assert_eq!(es_heading(0.0), 0);
    assert_eq!(es_heading(90.0), 1036);
    assert_eq!(es_heading(180.0), 2072);
    assert_eq!(es_heading(270.0), 3112);
}
