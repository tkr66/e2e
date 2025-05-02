use std::fs::File;
use std::io::Read;
use std::path::Path;

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct E2eYaml {
    pub driver: Driver,
    pub vars: Vars,
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
pub struct Vars(pub IndexMap<String, String>);

#[derive(Debug, Deserialize)]
pub struct Scenarios(pub IndexMap<String, Scenario>);

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Goto(String),
    Click(String),
    Focus(String),
    SendKeys {
        selector: String,
        value: String,
    },
    ScreenShot(String),
    WaitDisplayed {
        selector: String,
        timeout: u64,
        interval: u64,
    },
    AcceptAlert,
}

impl Step {
    pub fn expand_vars(&self, vars: &Vars) -> Self {
        match self {
            Step::Goto(url) => {
                let mut s = url.clone();
                for (k, v) in vars.0.iter() {
                    let key = format!("{{{}}}", k);
                    s = s.replace(key.as_str(), v);
                }
                Step::Goto(s)
            }
            Step::Click(selector) => {
                let mut s = selector.clone();
                for (k, v) in vars.0.iter() {
                    let key = format!("{{{}}}", k);
                    s = s.replace(key.as_str(), v);
                }
                Step::Click(s)
            }
            Step::Focus(selector) => {
                let mut s = selector.clone();
                for (k, v) in vars.0.iter() {
                    let key = format!("{{{}}}", k);
                    s = s.replace(key.as_str(), v);
                }
                Step::Focus(s)
            }
            Step::SendKeys { selector, value } => {
                let mut sel = selector.clone();
                let mut val = value.clone();
                for (k, v) in vars.0.iter() {
                    let key = format!("{{{}}}", k);
                    sel = sel.replace(key.as_str(), v);
                    val = val.replace(key.as_str(), v);
                }
                Step::SendKeys {
                    selector: sel,
                    value: val,
                }
            }
            Step::ScreenShot(path) => {
                let mut s = path.clone();
                for (k, v) in vars.0.iter() {
                    let key = format!("{{{}}}", k);
                    s = s.replace(key.as_str(), v);
                }
                Step::ScreenShot(s)
            }
            Step::WaitDisplayed {
                selector,
                timeout,
                interval,
            } => {
                let mut s = selector.clone();
                for (k, v) in vars.0.iter() {
                    let key = format!("{{{}}}", k);
                    s = s.replace(key.as_str(), v);
                }
                Step::WaitDisplayed {
                    selector: s,
                    timeout: *timeout,
                    interval: *interval,
                }
            }
            Step::AcceptAlert => Step::AcceptAlert,
        }
    }
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
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let yaml = "
driver:
  host: localhost
  port: 8080
  headless: true
  window:
    x: 0
    y: 0
    width: 1920
    height: 1080

vars:
  domain: en.wikipedia.org
  word: hello world
  css: '#id.class[attr=value]'
  img_out: ~/img
  timeout: 3000
  inteval: 500

scenarios:
  scenario1:
    name: index
    steps:
      - !goto 'https://{domain}/wiki/Main_Page'
      - !click '{css}'
      - !focus '{css}[name=btn]'
      - !send_keys { selector: 'input#searchInput', value: '{word}' }
      - !screen_shot '{img_out}/img0.png'
      - !wait_displayed { selector: '{css}', timeout: 3000, interval: 1000 }
        ";

        let v: E2eYaml = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(
            Some((0, &"domain".to_string(), &"en.wikipedia.org".to_string())),
            v.vars.0.get_full("domain")
        );
        assert_eq!(
            Some((1, &"word".to_string(), &"hello world".to_string())),
            v.vars.0.get_full("word")
        );
        assert_eq!(
            Some((2, &"css".to_string(), &"#id.class[attr=value]".to_string())),
            v.vars.0.get_full("css")
        );
        assert_eq!(
            Some((3, &"img_out".to_string(), &"~/img".to_string())),
            v.vars.0.get_full("img_out")
        );

        let scenario1 = v.scenarios.0.get("scenario1").unwrap();
        let steps: Vec<Step> = scenario1
            .steps
            .iter()
            .map(|s| s.expand_vars(&v.vars))
            .collect();
        let s1 = steps.first().unwrap();
        let s2 = steps.get(1).unwrap();
        let s3 = steps.get(2).unwrap();
        let s4 = steps.get(3).unwrap();
        let s5 = steps.get(4).unwrap();
        let s6 = steps.get(5).unwrap();
        assert_eq!(
            Step::Goto("https://en.wikipedia.org/wiki/Main_Page".to_string()),
            *s1
        );
        assert_eq!(Step::Click("#id.class[attr=value]".to_string()), *s2);
        assert_eq!(
            Step::Focus("#id.class[attr=value][name=btn]".to_string()),
            *s3
        );
        assert_eq!(
            Step::SendKeys {
                selector: "input#searchInput".to_string(),
                value: "hello world".to_string(),
            },
            *s4
        );
        assert_eq!(Step::ScreenShot("~/img/img0.png".to_string()), *s5);
        assert_eq!(
            Step::WaitDisplayed {
                selector: "#id.class[attr=value]".to_string(),
                timeout: 3000,
                interval: 1000,
            },
            *s6
        );
    }
}
