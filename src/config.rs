// tunable parameters

pub struct Config {
    // probe settings
    pub probe_timeout_ms: u64,
    pub probe_interval_secs: u64,

    // user location
    pub user_lat: f64,
    pub user_lon: f64,

    // latency color thresholds (absolute, in ms)
    pub green_below_ms: f64,
    pub yellow_below_ms: f64,
    // anything >= yellow_below_ms is red

    // map viewport (lon/lat bounds)
    pub map_lon_min: f64,
    pub map_lon_max: f64,
    pub map_lat_min: f64,
    pub map_lat_max: f64,
}

impl Config {
    pub fn default_canada() -> Self {
        Self {
            probe_timeout_ms: 500,
            probe_interval_secs: 3,

            user_lat: 43.65,
            user_lon: -79.38,

            green_below_ms: 30.0,
            yellow_below_ms: 80.0,

            map_lon_min: -145.0,
            map_lon_max: -45.0,
            map_lat_min: 38.0,
            map_lat_max: 65.0,
        }
    }
}
