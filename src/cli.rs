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

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_parse() {
        let args = ["e2e", "-f", "example-e2e.yaml", "s1", "s2"];
        let args: Args = Args::parse_from(args);

        assert_eq!(Some(vec!["s1".to_string(), "s2".to_string()]), args.names);
        assert_eq!(PathBuf::from("example-e2e.yaml"), args.file);
    }
}
