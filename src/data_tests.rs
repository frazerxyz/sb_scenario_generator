use std::{fs, path::PathBuf};

use crate::airport::{Airport, PositionRoute};

const DATA_DIR: &str = "data/airports";

fn airport_files() -> Vec<PathBuf> {
    fs::read_dir(DATA_DIR)
        .expect("Failed to read data directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect()
}

fn missing_references(route: &str, known: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    for token in route.split_whitespace() {
        if token.starts_with('$') {
            let name = &token[1..];
            if !known.iter().any(|k| k == name) {
                out.push(name.to_string());
            }
        }
    }
    out
}

#[test]
fn all_route_references_resolve() {
    let mut problems: Vec<String> = Vec::new();

    for path in airport_files() {
        let json = fs::read_to_string(&path).expect("could not read file");

        let airport: Airport = match serde_json::from_str(&json) {
            Ok(a) => a,
            Err(_) => continue,
        };

        let known: Vec<String> = airport
            .standard_routes
            .iter()
            .map(|r| r.name.clone())
            .collect();

        for dep in &airport.departure_routes {
            for name in missing_references(&dep.filed_route, &known) {
                problems.push(format!(
                    "{}: departure to {} references unknown standard route \"${name}\"",
                    path.display(),
                    dep.dest
                ));
            }
            for name in missing_references(&dep.flown_route, &known) {
                problems.push(format!(
                    "{}: departure to {} references unknown standard route \"${name}\"",
                    path.display(),
                    dep.dest
                ));
            }
        }

        for arr in &airport.arrival_routes {
            let mut routes = Vec::new();
            if let Some(r) = &arr.ctr_route {
                routes.push(r);
            }
            if let Some(r) = &arr.app_route {
                routes.push(r);
            }
            if let Some(r) = &arr.adc_route {
                routes.push(r);
            }

            for r in &routes {
                for name in missing_references(&r.flown_route, &known) {
                    problems.push(format!(
                        "{}: arrival from {} references unknown standard route \"${name}\"",
                        path.display(),
                        arr.dep
                    ));
                }
            }

            for name in missing_references(&arr.filed_route, &known) {
                problems.push(format!(
                    "{}: arrival from {} references unknown standard route \"${name}\"",
                    path.display(),
                    arr.dep
                ));
            }
        }
    }

    assert!(
        problems.is_empty(),
        "unresolved route references:\n{}",
        problems.join("\n")
    );
}

#[test]
fn all_airport_files_parse() {
    let mut problems: Vec<String> = Vec::new();

    let files = airport_files();
    assert!(!files.is_empty(), "no airport files found in {DATA_DIR}");

    for path in airport_files() {
        let json = match fs::read_to_string(&path) {
            Ok(j) => j,
            Err(e) => {
                problems.push(format!("{}: Could not read file: {e}", &path.display()));
                continue;
            }
        };
        if let Err(e) = serde_json::from_str::<Airport>(&json) {
            problems.push(format!("{}: {e}", &path.display()));
        }
    }

    assert!(
        problems.is_empty(),
        "airport data problems:\n{}",
        problems.join("\n")
    );
}

#[test]
fn spawn_coords_contains_colon() {
    let mut problems: Vec<String> = Vec::new();

    for path in airport_files() {
        let json = fs::read_to_string(&path).expect("could not read file");

        let airport: Airport = match serde_json::from_str(&json) {
            Ok(a) => a,
            Err(_) => continue,
        };

        for terminal in &airport.terminals {
            for stand in &terminal.stands {
                if !stand.coords.contains(':') {
                    problems.push(format!("{}: Stand number {} coordinates are incorectly formatted. Must be \"LATITUDE:LONGITUDE\"", path.display(), stand.id));
                }
            }
        }

        for runway in &airport.runways {
            if !runway.dep_spawn.contains(':') {
                problems.push(format!("{}: Runway {} spawn coordiantes are incorectly formatted. Must be \"LATITUDE:LONGITUDE\"", path.display(), runway.designator));
            }
        }

        let mut routes: Vec<PositionRoute> = Vec::new();

        for r in airport.arrival_routes {
            if let Some(r) = r.adc_route {
                routes.push(r);
            }
            if let Some(r) = r.app_route {
                routes.push(r);
            }
            if let Some(r) = r.ctr_route {
                routes.push(r);
            }
        }

        for r in routes {
            if !r.spawn_coords.contains(':') {
                problems.push(format!("{}: One or more position route spawn coordiantes are incorectly formatted. Must be \"LATITUDE:LONGITUDE\"", path.display()));
            }
        }

        if let Some(local) = airport.local_vfr {
            for a in local {
                if !a.spawn_coords.contains(':') {
                    problems.push(format!("{}: {} has incorrectly formatted spawn coordinates. Must be \"LATITUDE:LONGITUDE\"", path.display(), a.callsign));
                }
            }
        }

        if let Some(ground) = airport.ground_vfr {
            for c in ground.spawn_coords {
                if !c.contains(':') {
                    problems.push(format!("{}: One or more ground VFR spawn coordinates are formatted incorrectly. Must be \"LATITUDE:LONGITUDE\"", path.display()));
                }
            }
        }
    }

    assert!(
        problems.is_empty(),
        "airport data problems:\n{}",
        problems.join("\n")
    );
}
