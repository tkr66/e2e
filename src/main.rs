use std::path::PathBuf;

use clap::Parser;
use e2e_yaml::step::Step;
use thirtyfour::{ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};

mod e2e_yaml;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional list of scenario names to execute.
    /// If not provided, all scenarios will be run.
    #[arg(num_args = 1..)]
    names: Option<Vec<String>>,

    /// Path to the e2e_yaml configuration file.
    #[arg(short, long, default_value = "e2e.yaml")]
    file: PathBuf,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let e2e_yaml = e2e_yaml::load_e2e_yaml_from_file(&args.file)?;

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
    if let Some(names) = args.names {
        names.iter().for_each(|n| {
            if !e2e_yaml.scenarios.0.contains_key(n) {
                panic!(
                    "scenario '{}' is not defined in '{}'",
                    n,
                    args.file.to_str().unwrap()
                );
            }
        });

        for name in names {
            let scenario = e2e_yaml.scenarios.0.get(&name).unwrap();
            println!("running {}", scenario.name);
            let steps: Vec<Step> = scenario
                .steps
                .iter()
                .map(|s| s.expand_vars(&e2e_yaml.vars))
                .collect();
            for step in &steps {
                step.run(&driver, &e2e_yaml).await?;
            }
        }
    } else {
        for scenario in e2e_yaml.scenarios.0.values() {
            println!("running {}", scenario.name);
            let steps: Vec<Step> = scenario
                .steps
                .iter()
                .map(|s| s.expand_vars(&e2e_yaml.vars))
                .collect();
            for step in &steps {
                step.run(&driver, &e2e_yaml).await?;
            }
        }
    }

    driver.quit().await?;

    Ok(())
}
