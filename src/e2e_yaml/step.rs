use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::e2e_yaml::Vars;
use indexmap::IndexMap;
use serde::Deserialize;
use thirtyfour::error::WebDriverError;
use thirtyfour::extensions::query::*;
use thirtyfour::By;

use super::E2eYaml;
use crate::e2e_yaml::var::parse_var_names;

pub struct StepError {
    pub kind: StepErrorKind,
}

pub enum StepErrorKind {
    WebDriverError(WebDriverError),
    DirectoryCreateFailed(std::io::Error),
    AssertFailed(String, String),
    TaskNotFound(String),
}

impl From<WebDriverError> for StepError {
    fn from(err: WebDriverError) -> Self {
        Self {
            kind: StepErrorKind::WebDriverError(err),
        }
    }
}

impl From<std::io::Error> for StepError {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: StepErrorKind::DirectoryCreateFailed(err),
        }
    }
}

impl std::fmt::Display for StepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            StepErrorKind::WebDriverError(e) => writeln!(f, "{}", e),
            StepErrorKind::DirectoryCreateFailed(e) => writeln!(f, "{}", e),
            StepErrorKind::AssertFailed(expected, actual) => {
                writeln!(
                    f,
                    "\tassert failed. expected '{}', actual '{}'",
                    expected, actual
                )
            }
            StepErrorKind::TaskNotFound(id) => {
                writeln!(f, "task with id '{}' not found in configuration", id)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
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
    TaskRun {
        id: String,
        args: Option<Vec<String>>,
    },
    AssertEq {
        kind: ValueKind,
        expected: String,
        selector: String,
    },
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueKind {
    Text,
    Id,
    Class,
}

impl Step {
    pub fn expand_var(&self, name: &str, value: &str) -> Self {
        let key = format!("{{{}}}", name);
        let k = key.as_str();
        match self {
            Step::Goto(url) => Step::Goto(url.replace(k, value)),
            Step::Click(selector) => Step::Click(selector.replace(k, value)),
            Step::Focus(selector) => Step::Focus(selector.replace(k, value)),
            Step::SendKeys {
                selector,
                value: val,
            } => Step::SendKeys {
                selector: selector.replace(k, value),
                value: val.replace(k, value),
            },
            Step::ScreenShot(path) => Step::ScreenShot(path.replace(k, value)),
            Step::WaitDisplayed {
                selector,
                timeout,
                interval,
            } => Step::WaitDisplayed {
                selector: selector.replace(k, value),
                timeout: *timeout,
                interval: *interval,
            },
            Step::AcceptAlert => Step::AcceptAlert,
            Step::TaskRun { id, args } => {
                if let Some(args) = args {
                    let args: Vec<String> = args.iter().map(|arg| arg.replace(k, value)).collect();
                    Step::TaskRun {
                        id: id.clone(),
                        args: Some(args),
                    }
                } else {
                    Step::TaskRun {
                        id: id.clone(),
                        args: None,
                    }
                }
            }
            Step::AssertEq {
                kind,
                expected,
                selector,
            } => Step::AssertEq {
                kind: kind.clone(),
                expected: expected.replace(k, value),
                selector: selector.replace(k, value),
            },
        }
    }

    pub fn expand_vars(&self, vars: &Vars) -> Self {
        match self {
            Step::Goto(url) => Step::Goto(expand(url, vars)),
            Step::Click(selector) => Step::Click(expand(selector, vars)),
            Step::Focus(selector) => Step::Focus(expand(selector, vars)),
            Step::SendKeys { selector, value } => Step::SendKeys {
                selector: expand(selector, vars),
                value: expand(value, vars),
            },
            Step::ScreenShot(path) => Step::ScreenShot(expand(path, vars)),
            Step::WaitDisplayed {
                selector,
                timeout,
                interval,
            } => Step::WaitDisplayed {
                selector: expand(selector, vars),
                timeout: *timeout,
                interval: *interval,
            },
            Step::AcceptAlert => Step::AcceptAlert,
            Step::TaskRun { id, args } => {
                if let Some(args) = args {
                    let expanded: Vec<String> = args.iter().map(|x| expand(x, vars)).collect();
                    Step::TaskRun {
                        id: id.clone(),
                        args: Some(expanded),
                    }
                } else {
                    Step::TaskRun {
                        id: id.clone(),
                        args: None,
                    }
                }
            }
            Step::AssertEq {
                kind,
                expected,
                selector,
            } => Step::AssertEq {
                kind: kind.clone(),
                expected: expand(expected, vars),
                selector: expand(selector, vars),
            },
        }
    }

    pub async fn run(
        &self,
        driver: &thirtyfour::WebDriver,
        config: &E2eYaml,
    ) -> Result<(), StepError> {
        match self {
            Step::Goto(url) => driver.goto(url).await?,
            Step::Click(selector) => {
                let elem = driver.find(By::Css(selector)).await?;
                elem.click().await?;
            }
            Step::Focus(selector) => {
                let elem = driver.find(By::Css(selector)).await?;
                elem.focus().await?;
            }
            Step::SendKeys { selector, value } => {
                let elem = driver.find(By::Css(selector)).await?;
                elem.clear().await?;
                elem.send_keys(value).await?;
            }
            Step::ScreenShot(file_name) => {
                let p = Path::new(file_name);
                if let Some(dir) = p.parent() {
                    if !dir.exists() {
                        fs::create_dir_all(dir)?;
                    }
                }
                driver.screenshot(Path::new(file_name)).await?
            }
            Step::WaitDisplayed {
                selector,
                timeout,
                interval,
            } => {
                let elem = driver
                    .query(By::Css(selector))
                    .wait(
                        Duration::from_millis(*timeout),
                        Duration::from_millis(*interval),
                    )
                    .single()
                    .await?;
                elem.wait_until().displayed().await.map_err(|e| {
                    WebDriverError::Timeout(format!("selector: {}, {}", selector, e))
                })?;
            }
            Step::AcceptAlert => {
                driver.accept_alert().await?;
            }
            Step::TaskRun { id, args } => {
                let tasks = match &config.tasks {
                    Some(tasks) => tasks,
                    None => {
                        return Err(StepError {
                            kind: StepErrorKind::TaskNotFound(id.to_string()),
                        })
                    }
                };
                let t = match tasks.0.get(id) {
                    Some(task) => task,
                    None => {
                        return Err(StepError {
                            kind: StepErrorKind::TaskNotFound(id.to_string()),
                        })
                    }
                };
                let default_vars = Vars(IndexMap::new());
                let default_vars = match &config.vars {
                    Some(vars) => vars,
                    None => &default_vars,
                };
                let args: Option<Vec<&str>> = args
                    .as_ref()
                    .map(|x| x.iter().map(|y| y.as_str()).collect());
                let steps = t.expand_args(args.as_deref());
                let steps: Vec<Step> = steps
                    .into_iter()
                    .map(|x| x.expand_vars(default_vars))
                    .collect();
                for ele in steps {
                    Box::pin(ele.run(driver, config)).await?;
                }
            }
            Step::AssertEq {
                kind,
                expected,
                selector,
            } => {
                let elem = driver.find(By::Css(selector)).await?;
                let actual = match kind {
                    ValueKind::Text => elem.text().await?,
                    ValueKind::Id => elem.id().await?.unwrap_or("".to_string()),
                    ValueKind::Class => elem.class_name().await?.unwrap_or("".to_string()),
                };
                if expected != &actual {
                    return Err(StepError {
                        kind: StepErrorKind::AssertFailed(expected.to_string(), actual),
                    });
                }
            }
        }
        Ok(())
    }
}

fn expand(orig: &str, vars: &Vars) -> String {
    let mut result = orig.to_string();
    if let Some(names) = parse_var_names(orig) {
        for name in names {
            let key = format!("{{{}}}", name);
            let value = vars.0.get(name.as_str()).unwrap_or(&key);
            result = result.replace(key.as_str(), value);
        }
    };
    result
}

#[cfg(test)]
mod step_tests {
    use indexmap::IndexMap;

    use super::*;

    #[test]
    fn test_expand_vars() {
        let yaml = "
 - !goto '{url}'
 - !click '{app}'
 - !focus '{app}'
 - !send_keys { selector: '{app}', value: '{app}' }
 - !screen_shot '{app}'
 - !wait_displayed { selector: '{app}', timeout: 3000, interval: 1000 }
 - !task_run { id: login, args: [ 'admin', '{app}' ] }
 - !assert_eq { kind: text, expected: '{app}', selector: '{app}' }
";
        let vars = Vars(IndexMap::from([
            ("url".to_string(), "http://localhost".to_string()),
            ("app".to_string(), "e2e".to_string()),
        ]));
        let steps: Vec<Step> = serde_yaml::from_str(yaml).unwrap();
        let expanded_steps: Vec<Step> = steps.iter().map(|x| x.expand_vars(&vars)).collect();
        let s1 = &expanded_steps[0];
        let s2 = &expanded_steps[1];
        let s3 = &expanded_steps[2];
        let s4 = &expanded_steps[3];
        let s5 = &expanded_steps[4];
        let s6 = &expanded_steps[5];
        let s7 = &expanded_steps[6];
        let s8 = &expanded_steps[7];
        assert_eq!(Step::Goto("http://localhost".to_string()), *s1);
        assert_eq!(Step::Click("e2e".to_string()), *s2);
        assert_eq!(Step::Focus("e2e".to_string()), *s3);
        assert_eq!(
            Step::SendKeys {
                selector: "e2e".to_string(),
                value: "e2e".to_string(),
            },
            *s4
        );
        assert_eq!(Step::ScreenShot("e2e".to_string()), *s5);
        assert_eq!(
            Step::WaitDisplayed {
                selector: "e2e".to_string(),
                timeout: 3000,
                interval: 1000,
            },
            *s6
        );
        assert_eq!(
            Step::TaskRun {
                id: "login".to_string(),
                args: Some(vec!["admin".to_string(), "e2e".to_string()]),
            },
            *s7
        );
        assert_eq!(
            Step::AssertEq {
                kind: ValueKind::Text,
                expected: "e2e".to_string(),
                selector: "e2e".to_string(),
            },
            *s8
        );
    }
}
