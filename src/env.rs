use std::collections::HashMap;
use std::fs;

pub struct EnvVars {
    pub alpha_vantage_api_key: String,
}

impl EnvVars {
    pub fn load() -> Self {
        let contents = fs::read_to_string(".env").expect("Something went wrong reading the file");

        let map: HashMap<&str, &str> = contents
            .lines()
            .map(|line| line.split_once('=').unwrap())
            .collect();

        EnvVars {
            alpha_vantage_api_key: map.get("ALPHA_VANTAGE_API_KEY").unwrap().to_string(),
        }
    }
}
