use std::fs::File;
use std::io::Read;
use std::path::Path;

use driver::Driver;
use scenario::Scenarios;
use serde::Deserialize;
use task::Tasks;
use var::Vars;

pub mod driver;
pub mod scenario;
pub mod step;
pub mod task;
pub mod var;

#[derive(Debug, Deserialize)]
pub struct E2eYaml {
    pub driver: Driver,
    pub vars: Vars,
    pub tasks: Tasks,
    pub scenarios: Scenarios,
}

#[derive(Debug, Deserialize)]
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
