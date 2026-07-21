mod aircraft;
mod airport;
mod apc;
mod global;

use airport::Airport;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = fs::read_to_string("data/EGKK.json")?;

    let a: Airport = serde_json::from_str(&json)?;

    // println!("{:#?}", a);

    println!("{}\n", Airport::format_elevation(&a));
    println!("{}\n", Airport::format_runways(&a));
    println!("{}\n", Airport::format_holds(&a));

    Ok(())
}
