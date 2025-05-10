use std::fs::File;
use std::io::Read;
use std::path::Path;

use driver::Driver;
use scenario::Scenarios;
use serde::Deserialize;
use serde::Serialize;
use task::Tasks;
use var::Vars;

pub mod driver;
pub mod scenario;
pub mod step;
pub mod task;
pub mod var;

#[derive(Debug, Deserialize, Serialize)]
pub struct E2eYaml {
    pub driver: Driver,
    pub vars: Option<Vars>,
    pub tasks: Option<Tasks>,
    pub scenarios: Scenarios,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Window {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
}

pub fn load_e2e_yaml_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<E2eYaml, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

#[cfg(test)]
mod e2e_yaml_tests {
    use super::*;

    #[test]
    fn test_minimal() {
        let yaml = "
driver:
  host: localhost
  port: 4444
  headless: true
  window:
    x: 0
    y: 0
    width: 1920
    height: 1080

scenarios:
  s1:
    name: first
    steps:
      - !goto www.google.com
";
        let _: E2eYaml = serde_yaml::from_str(yaml).unwrap();
    }
}
