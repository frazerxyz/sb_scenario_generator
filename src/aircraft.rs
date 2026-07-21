use rand::seq::SliceRandom;

use crate::aircraft::{
    FlightRule::{I, V},
    FlightType::{Arrival, Departure},
};

pub enum FlightRule {
    I,
    V,
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
    squawk: Option<u16>,
    spawn_coords: String,
    spawn_altitude: u16,
    spawn_hdg: Option<u16>,
    filed_route: String,
    tas: Option<u16>,
    rfl: Option<u16>,
    flown_route: String,
    start: u16,
    delay: Option<[u16; 2]>,
    initial_pseudo_pilot: Option<String>,
}

// impl fmt::Display for Aircraft {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "@N:{}:{}:1:{}:{}:0:{}:0", self.callsign,)
//     }
// }

pub fn string_if_none<T: std::fmt::Display>(input: Option<T>, none: &str) -> String {
    match input {
        Some(s) => format!("{s}"),
        None => none.to_string(),
    }
}

struct SquawkPool {
    available: Vec<u16>,
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
