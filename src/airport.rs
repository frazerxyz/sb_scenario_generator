use crate::controller::Controller;
use crate::route::Route;

pub struct Hold {
    pub fix: String,
    pub course: u16,
    pub direction: i8,
}

impl Hold {
    pub fn new(fix: &str, course: u16, direction: i8) -> Hold {
        Hold {
            fix: fix.to_string(),
            course,
            direction,
        }
    }
}

pub struct Runway {
    pub number: String,
    pub coords: String,
}

impl Runway {
    pub fn new(number: &str, coords: &str) -> Runway {
        Runway {
            number: number.to_string(),
            coords: coords.to_string(),
        }
    }
}

pub struct Airport {
    pub icao: String,
    pub altitude: f64,
    pub runways: Vec<Runway>,
    pub holds: Vec<Hold>,
    pub routes: Vec<Route>,
    pub controllers: Vec<Controller>,
}

impl Airport {
    pub fn new(
        icao: &str,
        altitude: f64,
        runways: Vec<Runway>,
        holds: Vec<Hold>,
        routes: Vec<Route>,
        controllers: Vec<Controller>,
    ) -> Airport {
        Airport {
            icao: icao.to_string(),
            altitude,
            runways,
            holds,
            routes,
            controllers,
        }
    }
}
