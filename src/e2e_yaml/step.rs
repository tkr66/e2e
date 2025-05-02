use std::path::Path;
use std::time::Duration;
use std::fs;

use crate::e2e_yaml::Vars;
use serde::Deserialize;
use thirtyfour::error::WebDriverError;
use thirtyfour::extensions::query::*;
use thirtyfour::By;

use super::E2eYaml;

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
                    let args: Vec<String> =
                        args.iter().map(|arg| arg.replace(k, value)).collect();
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
            Step::TaskRun { id, args } => {
                if let Some(args) = args {
                    let mut expanded_args: Vec<String> = Vec::new();
                    for arg in args {
                        let mut expanded_arg = arg.clone();
                        for (k, v) in vars.0.iter() {
                            let key = format!("{{{}}}", k);
                            expanded_arg = expanded_arg.replace(key.as_str(), v);
                        }
                        expanded_args.push(expanded_arg);
                    }
                    Step::TaskRun {
                        id: id.clone(),
                        args: Some(expanded_args),
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
