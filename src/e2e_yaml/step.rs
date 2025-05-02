use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::e2e_yaml::Vars;
use serde::Deserialize;
use thirtyfour::error::WebDriverError;
use thirtyfour::extensions::query::*;
use thirtyfour::By;

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

    pub async fn run(
        &self,
        driver: &thirtyfour::WebDriver,
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
        }
        Ok(())
    }
}
