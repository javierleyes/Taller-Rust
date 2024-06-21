use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::result::Result;
/// This struct represents a config error
#[derive(Debug, Clone)]
pub struct ConfigError;

/// This struct represents a thread pool
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error creating config")
    }
}

pub struct Config {
    pub port: String,
    pub log_file: String,
}

impl Config {
    pub fn from_file(filename: &str) -> Result<Self, ConfigError> {
        let file = File::open(filename).expect("Failed to read config file");
        let reader = BufReader::new(file);
        let config_entries = Config::read_entries(reader);
        let port = config_entries.get("port").expect("No port was provided");
        let log_file = config_entries
            .get("logFile")
            .expect("No logFile was provided");
        Ok(Config {
            port: port.to_string(),
            log_file: log_file.to_string(),
        })
    }

    fn read_entries(reader: BufReader<File>) -> HashMap<String, String> {
        let mut entries: HashMap<String, String> = HashMap::new();
        for (_, read_line) in reader.lines().enumerate() {
            if let Ok(line) = read_line {
                let split_line: Vec<&str> = line.split('=').collect();
                let key = split_line[0];
                let value = split_line[1];
                entries.insert(key.to_string(), value.to_string());
            }
        }
        entries
    }
}
