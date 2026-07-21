use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Runway {
    pub designator: String,
    pub threshold: String,
    pub track: f32,
    pub dep_spawn: String,
}

#[derive(Debug, Deserialize)]
pub enum Direction {
    L,
    R,
}

impl Direction {
    pub fn to_i8(&self) -> i8 {
        match self {
            Direction::L => -1,
            Direction::R => 1,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Hold {
    pub fix: String,
    pub inbound: u16,
    pub direction: Direction,
}

#[derive(Debug, Deserialize)]
pub struct CustomRoute {
    pub name: String,
    pub route: String,
}

#[derive(Debug, Deserialize)]
pub struct Controller {
    pub callsign: String,
    pub frequency: f32,
}

#[derive(Debug, Deserialize)]
pub struct Airline {
    pub icao: String,
    pub terminal: Vec<u8>,
    pub types: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum ReferenceCode {
    A,
    B,
    C,
    D,
    E,
    F,
}

#[derive(Debug, Deserialize)]
pub struct Stand {
    pub id: u32,
    pub coords: String,
    pub bearing: f32,
    pub max_size: ReferenceCode,
}

#[derive(Debug, Deserialize)]
pub struct Terminal {
    pub id: u32,
    pub label: String,
    pub stands: Vec<Stand>,
}

#[derive(Debug, Deserialize)]
pub struct RunwayRoute {
    pub runway: Option<String>,
    pub filed_route: String,
    pub flown_route: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StandardRoute {
    pub name: String,
    pub route: Vec<RunwayRoute>,
}

#[derive(Debug, Deserialize)]
pub struct DepartureRoute {
    pub dest: String,
    pub callsigns: Vec<String>,
    pub types: Vec<String>,
    pub filed_route: String,
    pub flown_route: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PositionRoute {
    pub spawn_coords: String,
    pub spawn_alt: u16,
    pub flown_route: String,
}

#[derive(Debug, Deserialize)]
pub struct ArrivalRoute {
    pub dep: String,
    pub callsigns: Vec<String>,
    pub types: Vec<String>,
    pub filed_route: String,
    pub ctr_route: Option<PositionRoute>,
    pub app_route: Option<PositionRoute>,
    pub adc_route: Option<PositionRoute>,
}

#[derive(Debug, Deserialize)]
pub struct Airport {
    pub icao: String,
    pub elevation: f32,
    pub runways: Vec<Runway>,
    pub holds: Vec<Hold>,
    pub custom_routes: Vec<CustomRoute>,
    pub controllers: Vec<Controller>,
    pub airlines: Vec<Airline>,
    pub terminals: Vec<Terminal>,
    pub standard_routes: Vec<StandardRoute>,
    pub departure_routes: Vec<DepartureRoute>,
    pub arrival_routes: Vec<ArrivalRoute>,
}

impl Airport {
    pub fn format_elevation(&self) -> String {
        format!("AIRPORT_ALT:{:.1}", self.elevation)
    }
    pub fn format_holds(&self) -> String {
        self.holds
            .iter()
            .map(|h| format!("HOLDING:{}:{}:{}", h.fix, h.inbound, h.direction.to_i8()))
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn format_runways(&self) -> String {
        self.runways
            .iter()
            .map(|r| format!("ILS{}:{}:{}", r.designator, r.threshold, r.track))
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn format_custom_routes(&self) -> String {
        self.custom_routes
            .iter()
            .map(|r| format!("ROUTE:{}:{}", r.name, r.route))
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn format_controllers(&self) -> String {
        self.controllers
            .iter()
            .map(|c| {
                format!(
                    "PSEUDOPILOT:ALL\nCONTROLLER:{}:{:.3}",
                    c.callsign, c.frequency
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
