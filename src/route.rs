pub struct Route {
    pub name: String,
    pub route: String,
}

impl Route {
    pub fn new(name: &str, route: &str) -> Route {
        Route {
            name: name.to_string(),
            route: route.to_string(),
        }
    }
}
