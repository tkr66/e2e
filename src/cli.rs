use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file.
    #[arg(short, long, default_value = "e2e.yaml")]
    pub file: PathBuf,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, PartialEq, Debug)]
pub enum Cmd {
    /// Executes scenarios
    Run(RunArgs),

    /// Print parsed e2e-yaml file
    Config(ConfigArgs),
}

#[derive(Parser, PartialEq, Debug)]
pub struct RunArgs {
    /// Optional list of scenario names to execute.
    /// If not provided, all scenarios will be run.
    #[arg(num_args = 1..)]
    pub names: Option<Vec<String>>,
}

#[derive(Parser, PartialEq, Debug)]
pub struct ConfigArgs {
    /// Specifies a specific configuration section to display.
    /// If omitted, the entire configuration is displayed.
    pub key: Option<ConfigSection>,
}

#[derive(ValueEnum, PartialEq, Clone, Debug)]
pub enum ConfigSection {
    Driver,
    Vars,
    Tasks,
    Scenarios,
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_parse_run() {
        let args: Args = Args::parse_from(["e2e", "run"]);
        assert_eq!(PathBuf::from("e2e.yaml"), args.file);
        assert_eq!(Cmd::Run(RunArgs { names: None }), args.cmd);

        let args: Args = Args::parse_from(["e2e", "run", "s1", "s2"]);
        assert_eq!(PathBuf::from("e2e.yaml"), args.file);
        assert_eq!(
            Cmd::Run(RunArgs {
                names: Some(vec!["s1".to_string(), "s2".to_string()])
            }),
            args.cmd
        );
    }

    #[test]
    fn test_parse_config() {
        let args: Args = Args::parse_from(["e2e", "config"]);
        assert_eq!(PathBuf::from("e2e.yaml"), args.file);
        assert_eq!(Cmd::Config(ConfigArgs { key: None }), args.cmd);

        let args: Args = Args::parse_from(["e2e", "config", "driver"]);
        assert_eq!(PathBuf::from("e2e.yaml"), args.file);
        assert_eq!(
            Cmd::Config(ConfigArgs {
                key: Some(ConfigSection::Driver)
            }),
            args.cmd
        );
    }
}
