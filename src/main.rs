use std::process;

use clap::Parser;
use cli::Args;
use e2e_yaml::{step::Step, task::Tasks, var::Vars};
use indexmap::IndexMap;

mod cli;
mod e2e_yaml;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let e2e_yaml = e2e_yaml::load_e2e_yaml_from_file(&args.file)?;

    if let Some(Err(e)) = e2e_yaml
        .tasks
        .as_ref()
        .map(Tasks::detect_circular_dependencies)
    {
        eprintln!("{}", e);
        process::exit(1);
    }

    match &args.cmd {
        cli::Cmd::Run(args) => {
            let scenarios = if let Some(names) = &args.names {
                let names_ref: Vec<&str> = names.iter().map(|x| x.as_str()).collect();
                e2e_yaml.scenarios.find(&names_ref)?
            } else {
                e2e_yaml.scenarios.0.values().collect()
            };

            let default_vars = Vars(IndexMap::new());
            let default_vars = match &e2e_yaml.vars {
                Some(v) => v,
                None => &default_vars,
            };
            let driver = e2e_yaml.driver.initialize().await?;
            for scenario in scenarios {
                println!("running {}", scenario.name);
                let steps: Vec<Step> = scenario
                    .steps
                    .iter()
                    .map(|s| s.expand_vars(default_vars))
                    .collect();
                for step in &steps {
                    if let Err(err) = step.run(&driver, &e2e_yaml).await {
                        eprintln!("{}", err);
                        break;
                    };
                }
            }

            driver.quit().await?;
        }
        cli::Cmd::Config(_) => {
            println!("{}", serde_yaml::to_string(&e2e_yaml).unwrap());
        }
    }

    Ok(())
}
