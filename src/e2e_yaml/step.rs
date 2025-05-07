use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::e2e_yaml::Vars;
use serde::Deserialize;
use thirtyfour::error::WebDriverError;
use thirtyfour::extensions::query::*;
use thirtyfour::By;

use super::{parse_var_names, E2eYaml};

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
    TaskRun {
        id: String,
        args: Option<Vec<String>>,
    },
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
        }
    }

    pub async fn run(
        &self,
        driver: &thirtyfour::WebDriver,
        config: &E2eYaml,
    ) -> Result<(), thirtyfour::error::WebDriverError> {
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
                let t = config.tasks.0.get(id).unwrap();
                t.run(driver, config, args.as_ref()).await?;
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
