use crate::{
    airport::{RouteEntry, StandardRoute},
    config_error,
};

pub enum RouteType {
    Filed,
    Flown,
}

pub fn lookup(
    name: &str,
    route_type: &RouteType,
    runway: &str,
    standard_routes: &[StandardRoute],
) -> String {
    let selected_route = standard_routes.iter().find(|r| r.name == name);
    match selected_route {
        Some(s) => find_runway_route(&s.routes, runway, route_type),
        None => config_error(&format!("no standard route named \"{name}\"")),
    }
}

pub fn find_runway_route(
    route_entries: &[RouteEntry],
    runway: &str,
    route_type: &RouteType,
) -> String {
    match route_entries.iter().find(|e| match &e.runway {
        None => true,
        Some(r) => r == runway,
    }) {
        Some(r) => r.route(route_type).to_string(),
        None => config_error(&format!("standard route could not be found for {runway}")),
    }
}

pub fn route_parser(
    original_route: &str,
    standard_routes: &[StandardRoute],
    runway: &str,
    route_type: &RouteType,
) -> String {
    let mut output: Vec<String> = Vec::new();

    for token in original_route.split_whitespace() {
        if token.starts_with('$') {
            let name = &token[1..];
            output.push(lookup(name, route_type, runway, standard_routes));
        } else {
            output.push(token.to_string());
        }
    }

    output.join(" ")
}
