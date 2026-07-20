mod airport_parser;

use airport_parser::Airport;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = fs::read_to_string("data/EGKK.json")?;

    let a: Airport = serde_json::from_str(&json)?;

    println!("{:#?}", a);

    Ok(())
}
