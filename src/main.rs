use std::process::{self, ExitCode};

use clap::Parser;
use cli::Args;
use e2e_yaml::task::Tasks;

mod cli;
mod e2e_yaml;

#[tokio::main]
async fn main() -> std::result::Result<ExitCode, Box<dyn std::error::Error>> {
    let args = Args::parse();
    let e2e_yaml = e2e_yaml::load_e2e_yaml_from_file(&args.file)?.expand();

    if let Some(Err(e)) = e2e_yaml
        .tasks
        .as_ref()
        .map(Tasks::detect_circular_dependencies)
    {
        eprintln!("{}", e);
        process::exit(1);
    }

    let exit_code = args.cmd.run(e2e_yaml).await?;

    Ok(ExitCode::from(exit_code))
}
