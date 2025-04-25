use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct E2eYaml {
    pub driver: Driver,
    pub scenarios: Scenarios,
}

#[derive(Debug, Deserialize)]
pub struct Driver {
    pub host: String,
    pub port: String,
    pub headless: bool,
    pub window: Window,
}

#[derive(Debug, Deserialize)]
pub struct Window {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct Scenarios(pub HashMap<String, Scenario>);

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Goto(String),
    Click(String),
    Focus(String),
    SendKeys { selector: String, value: String },
    ScreenShot(String),
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
