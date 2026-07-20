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
    icao: String,
    elevation: f32,
    runways: Vec<Runway>,
    holds: Vec<Hold>,
    custom_routes: Vec<CustomRoute>,
    controllers: Vec<Controller>,
    airlines: Vec<Airline>,
    terminals: Vec<Terminal>,
    standard_routes: Vec<StandardRoute>,
    departure_routes: Vec<DepartureRoute>,
    arrival_routes: Vec<ArrivalRoute>,
}
