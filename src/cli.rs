use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Optional list of scenario names to execute.
    /// If not provided, all scenarios will be run.
    #[arg(num_args = 1..)]
    pub names: Option<Vec<String>>,

    /// Path to the e2e_yaml configuration file.
    #[arg(short, long, default_value = "e2e.yaml")]
    pub file: PathBuf,
}
