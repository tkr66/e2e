use std::time::Duration;
use std::{env::args, path::Path};

use thirtyfour::error::WebDriverError;
use thirtyfour::extensions::query::*;
use thirtyfour::{By, ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};

mod args;
mod e2e_yaml;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = args::parse_args(args().collect())?;
    let e2e_yaml = e2e_yaml::load_e2e_yaml_from_file(args.e2e_yaml())?;

    let mut caps = DesiredCapabilities::edge();
    if e2e_yaml.driver.headless {
        caps.set_headless()?;
    }
    let driver_url = format!("http://{}:{}", &e2e_yaml.driver.host, &e2e_yaml.driver.port);
    let driver = WebDriver::new(driver_url, caps).await?;
    let window = &e2e_yaml.driver.window;
    driver
        .set_window_rect(window.x, window.y, window.width, window.height)
        .await?;

    for scenario in e2e_yaml.scenarios.0.values() {
        println!("running {}", scenario.name);
        for step in &scenario.steps {
            match step {
                e2e_yaml::Step::Goto(url) => driver.goto(url).await?,
                e2e_yaml::Step::Click(selector) => {
                    let elem = driver.find(By::Css(selector)).await?;
                    elem.click().await?;
                }
                e2e_yaml::Step::Focus(selector) => {
                    let elem = driver.find(By::Css(selector)).await?;
                    elem.focus().await?;
                }
                e2e_yaml::Step::SendKeys { selector, value } => {
                    let elem = driver.find(By::Css(selector)).await?;
                    elem.send_keys(value).await?;
                }
                e2e_yaml::Step::ScreenShot(file_name) => {
                    driver.screenshot(Path::new(file_name)).await?
                }
                e2e_yaml::Step::WaitDisplayed {
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
                e2e_yaml::Step::AcceptAlert => {
                    driver.accept_alert().await?;
                }
            }
        }
    }

    driver.quit().await?;

    Ok(())
}
