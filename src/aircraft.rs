use rand::seq::SliceRandom;
use std::fmt;

use crate::aircraft::{
    FlightRule::{I, V},
    FlightType::{Arrival, Departure},
};

pub enum FlightRule {
    I,
    V,
}

impl fmt::Display for FlightRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r = match self {
            I => "I".to_string(),
            V => "V".to_string(),
        };
        write!(f, "{r}")
    }
}

pub enum FlightType {
    Departure,
    Arrival,
    Local,
    Overflight,
}

pub struct Aircraft {
    flight_type: FlightType,
    flight_rule: FlightRule,
    callsign: String,
    aircraft_type: String,
    squawk: Option<u16>,
    spawn_coords: String,
    spawn_altitude: u16,
    spawn_hdg: Option<u16>,
    origin: String,
    dest: String,
    filed_route: String,
    tas: Option<u16>,
    rfl: Option<u16>,
    flown_route: String,
    start: u16,
    delay: Option<[u16; 2]>,
    initial_pseudo_pilot: String,
}

impl fmt::Display for Aircraft {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let aircraft_position = format!(
            "@N:{}:{}:1:{}:{}:0:{}:0",
            self.callsign,
            string_if_none(self.squawk, ""),
            self.spawn_coords,
            self.spawn_altitude,
            string_if_none(self.spawn_hdg, "")
        );
        let flight_plan = format!(
            "$FP{}:*A:{}:{}:{}:{}:0000:0000:{}:{}:00:00:0:0::/v/:{}",
            self.callsign,
            self.flight_rule,
            self.aircraft_type,
            string_if_none(self.tas, "0"),
            self.origin,
            string_if_none(self.rfl, "2000"),
            self.dest,
            self.filed_route
        );
        let route = format!(
            "$ROUTE:{}:{}:\nSTART:{}",
            self.callsign, self.flown_route, self.start
        );

        write!(
            f,
            "{aircraft_position}\n{flight_plan}\n{route}\n{}",
            self.initial_pseudo_pilot
        )
    }
}

pub fn string_if_none<T: std::fmt::Display>(input: Option<T>, none: &str) -> String {
    match input {
        Some(s) => format!("{s}"),
        None => none.to_string(),
    }
}

pub struct SquawkPool {
    pub available: Vec<u16>,
}

impl SquawkPool {
    fn new(rng: &mut impl rand::Rng) -> SquawkPool {
        let mut available: Vec<u16> = (0o4000..0o7000).collect();
        available.shuffle(rng);
        SquawkPool { available }
    }

    fn allocate(&mut self) -> Option<u16> {
        self.available.pop()
    }
}

pub fn allocate_squawk(a: &Aircraft) -> u16 {
    match (&a.flight_rule, &a.flight_type) {
        (V, _) => 7000,
        (I, Departure) => 2000,
        (I, Arrival) => 1234,
        (I, _) => 2000,
    }
}
