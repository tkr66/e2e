use std::fs::File;
use std::io::Read;
use std::path::Path;

use driver::Driver;
use indexmap::IndexMap;
use scenario::Scenarios;
use serde::Deserialize;
use serde::Serialize;
use step::Step;
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

impl E2eYaml {
    pub fn expand(self) -> Self {
        let mut scenarios = self.scenarios;
        let m = &mut scenarios.0;

        let default_vars = Vars(IndexMap::new());
        let default_vars = match &self.vars {
            Some(v) => v,
            None => &default_vars,
        };

        for scenario in m.values_mut() {
            let mut steps: Vec<Step> = Vec::new();
            scenario
                .steps
                .iter()
                .for_each(|x| steps.push(x.expand_vars(default_vars)));
            scenario.steps = steps;
        }

        Self {
            driver: self.driver,
            vars: self.vars,
            tasks: self.tasks,
            scenarios,
        }
    }
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
    use scenario::Scenario;

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

    #[test]
    fn test_expand() {
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

vars:
  a: hello
  b: world

scenarios:
  s1:
    name: first
    steps:
      - !goto 'www.google.com?search?q={a} {b}'
";
        let config: E2eYaml = serde_yaml::from_str(yaml).unwrap();
        let expanded_config = config.expand();
        assert_eq!(
            Scenarios(IndexMap::from([(
                "s1".to_string(),
                Scenario {
                    name: "first".to_string(),
                    steps: vec![Step::Goto(
                        "www.google.com?search?q=hello world".to_string()
                    )]
                }
            )])),
            expanded_config.scenarios
        );
    }
}
