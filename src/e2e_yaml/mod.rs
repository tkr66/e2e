use std::fs::File;
use std::io::Read;
use std::path::Path;

use driver::Driver;
use indexmap::IndexMap;
use scenario::Scenarios;
use serde::Deserialize;
use task::Tasks;

pub mod driver;
pub mod scenario;
pub mod step;
pub mod task;

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

#[derive(Debug, Deserialize)]
pub struct Vars(pub IndexMap<String, String>);

pub fn load_e2e_yaml_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<E2eYaml, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

pub fn parse_var_names(input: &str) -> Option<Vec<String>> {
    let mut buf: Option<String> = None;
    let mut result: Vec<String> = Vec::new();
    let mut peekable = input.chars().peekable();
    while let Some(c) = peekable.next() {
        match c {
            '{' => {
                // escape
                if let Some(&'{') = peekable.peek() {
                    peekable.next();
                    continue;
                }
                buf = Some(String::new());
            }
            '}' => {
                // escape
                if let Some(&'}') = peekable.peek() {
                    peekable.next();
                    continue;
                }
                if buf.is_some() {
                    result.push(buf.clone().unwrap());
                }
            }
            _ => {
                if let Some(ref mut b) = buf {
                    b.push(c);
                }
            }
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use step::{Step, ValueKind};
    use task::Task;

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

tasks:
  login:
    arg_names:
      - name
      - password
    steps:
      - !goto '{root}/enwiki/wiki/Special:UserLogin'
      - !send_keys { selector: '#wpName1', value: '{name}' }
      - !send_keys { selector: '#wpPassword', value: '{password}' }
      - !click '#wpLoginAttempt'

  login2:
    steps:
      - !goto 'a'

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
      - !task_run { id: login, args: [ 'admin', 'password' ] }
      - !assert_eq { kind: text, expected: abc, selector: '{css}' }
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

        let t1 = v.tasks.0.get("login").unwrap();
        let t2 = v.tasks.0.get("login2").unwrap();
        assert_eq!(
            Task {
                arg_names: Some(vec!["name".to_string(), "password".to_string()]),
                steps: vec![
                    Step::Goto("{root}/enwiki/wiki/Special:UserLogin".to_string()),
                    Step::SendKeys {
                        selector: "#wpName1".to_string(),
                        value: "{name}".to_string()
                    },
                    Step::SendKeys {
                        selector: "#wpPassword".to_string(),
                        value: "{password}".to_string()
                    },
                    Step::Click("#wpLoginAttempt".to_string()),
                ]
            },
            *t1
        );
        assert_eq!(
            Task {
                arg_names: None,
                steps: vec![Step::Goto("a".to_string()),]
            },
            *t2
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
        let s7 = steps.get(6).unwrap();
        let s8 = &steps[7];
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
        assert_eq!(
            Step::TaskRun {
                id: "login".to_string(),
                args: Some(vec!["admin".to_string(), "password".to_string()]),
            },
            *s7
        );
        assert_eq!(
            Step::AssertEq {
                kind: ValueKind::Text,
                expected: "abc".to_string(),
                selector: "#id.class[attr=value]".to_string(),
            },
            *s8
        );
    }

    #[test]
    fn test_single_variable() {
        let input = "{name}";
        let expected = Some(vec!["name".to_string()]);
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_multiple_variables() {
        let input = "{a} {b} {c} {{{d}}}";
        let expected = Some(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]);
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_escaped_braces() {
        let input = "{{not_a_var}} and {real}";
        let expected = Some(vec!["real".to_string()]);
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let expected = None;
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_no_variables() {
        let input = "hello world";
        let expected = None;
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_trailing_brace_without_opening() {
        let input = "oops}";
        let expected = None;
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_escape_sequence_only() {
        let input = "{{}}";
        let expected = None;
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_whitespace_inside_var() {
        let input = "{hello world}";
        let expected = Some(vec!["hello world".to_string()]);
        assert_eq!(parse_var_names(input), expected);
    }
}
