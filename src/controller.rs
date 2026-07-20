pub struct Controller {
    pub callsign: String,
    pub frequency: String,
}

impl Controller {
    pub fn new(callsign: &str, frequency: &str) -> Controller {
        Controller {
            callsign: callsign.to_string(),
            frequency: frequency.to_string(),
        }
    }
}
