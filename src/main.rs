use std::env::args;

use e2e_yaml::step::Step;
use thirtyfour::{ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};

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
        let steps: Vec<Step> = scenario
            .steps
            .iter()
            .map(|s| s.expand_vars(&e2e_yaml.vars))
            .collect();
        for step in &steps {
            step.run(&driver).await?;
        }
    }

    driver.quit().await?;

    Ok(())
}
