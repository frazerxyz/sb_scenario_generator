use std::fs;

struct Hold {
    fix: String,
    course: u16,
    direction: i8,
}

impl Hold {
    fn new(fix: &str, course: u16, direction: i8) -> Hold {
        Hold {
            fix: fix.to_string(),
            course,
            direction,
        }
    }
}

struct Runway {
    number: String,
    coords: String,
}

impl Runway {
    fn new(number: &str, coords: &str) -> Runway {
        Runway {
            number: number.to_string(),
            coords: coords.to_string(),
        }
    }
}

struct Route {
    name: String,
    route: String,
}

impl Route {
    fn new(name: &str, route: &str) -> Route {
        Route {
            name: name.to_string(),
            route: route.to_string(),
        }
    }
}

struct Controller {
    callsign: String,
    frequency: String,
}

impl Controller {
    fn new(callsign: &str, frequency: &str) -> Controller {
        Controller {
            callsign: callsign.to_string(),
            frequency: frequency.to_string(),
        }
    }
}

struct Airport {
    icao: String,
    altitude: f64,
    runways: Vec<Runway>,
    holds: Vec<Hold>,
    routes: Vec<Route>,
    controllers: Vec<Controller>,
}

impl Airport {
    fn new(
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

enum FlightType {
    Departure,
    Arrival,
    Vfr
}

fn runways(r: &[Runway]) -> String {
    r.iter()
        .map(|r| format!("ILS{}:{}", r.number, r.coords))
        .collect::<Vec<_>>()
        .join("\n")
}

fn holds(h: &[Hold]) -> String {
    h.iter()
        .map(|h| format!("HOLDING:{}:{}:{}", h.fix, h.course, h.direction))
        .collect::<Vec<_>>()
        .join("\n")
}

fn routes(r: &[Route]) -> String {
    r.iter()
        .map(|r| format!("ROUTE:{}:{}", r.name, r.route))
        .collect::<Vec<_>>()
        .join("\n")
}

fn controllers(c: &[Controller]) -> String {
    c.iter()
        .map(|c| format!("PSEUDOPILOT:ALL\nCONTROLLER:{}:{}", c.callsign, c.frequency))
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user_runways = vec![
        Runway::new("26L", "51.1515041:-0.1660353:51.1458769:-0.2067485"),
        Runway::new("08R", "51.1451795:-0.2120965:51.1514852:-0.1659789"),
    ];

    let user_holds: Vec<Hold> = vec![
        Hold::new("TIMBA", 308, 1),
        Hold::new("WILLO", 283, -1),
        Hold::new("MAY", 88, -1),
    ];

    let user_routes: Vec<Route> = vec![
        Route::new(
            "VFR Left Hand",
            "KK26LUC/1500 KK26LCD/1500 KK26LDB/1100 KK26LBF/800 ILS26L",
        ),
        Route::new("RNP26L", "OLEVI/3000 K26LF ILS26L"),
    ];

    let user_controllers: Vec<Controller> = vec![
        Controller::new("EGKK_N_GND", "121.540"),
        Controller::new("EGKK_APP", "126.825"),
        Controller::new("LTC_S_CTR", "134.125"),
    ];

    let a = Airport::new(
        "EGKK",
        202.0,
        user_runways,
        user_holds,
        user_routes,
        user_controllers,
    );

    let output: String = format!(
        "PSEUDOPILOT:ALL\n\nAIRPORT_ALT:{:.1}\n\n{}\n\n{}\n\n{}\n\n{}",
        a.altitude,
        runways(&a.runways),
        holds(&a.holds),
        routes(&a.routes),
        controllers(&a.controllers)
    );

    fs::write("output.txt", output)?;

    println!("Generated sweatbox scenario file for {}", a.icao);

    Ok(())
}
