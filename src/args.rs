#[derive(Debug, PartialEq, Eq)]
pub enum ParseArgsError {
    Unknown(String),
}

impl std::error::Error for ParseArgsError {}

impl std::fmt::Display for ParseArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Args {
    e2e_yaml: Option<String>,
}

impl Args {
    pub fn e2e_yaml(&self) -> &str {
        self.e2e_yaml.as_deref().unwrap_or("e2e.yaml")
    }
}

pub fn parse_args(args: Vec<String>) -> std::result::Result<Args, ParseArgsError> {
    let mut parsed_args = Args::default();
    let mut i = 1;
    while let Some(x) = args.get(i) {
        i += 1;
        match x.as_str() {
            "-f" | "--file" => {
                parsed_args.e2e_yaml = args.get(i).cloned();
                i += 1;
            }
            _ => return Err(ParseArgsError::Unknown("".to_string())),
        };
    }

    Ok(parsed_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_args() {
        let args = vec!["bin".to_string(), "a".to_string()];

        let args = parse_args(args);
        assert!(args.is_err());
        assert_eq!(args.err(), Some(ParseArgsError::Unknown("".to_string())))
    }

    #[test]
    fn test_default_args() {
        let args = vec![];

        let args = parse_args(args).unwrap();
        assert_eq!(args.e2e_yaml(), "e2e.yaml");
    }

    #[test]
    fn test_parse_args() {
        let args = vec![
            "e2e".to_string(),
            "-f".to_string(),
            "example-e2e.yaml".to_string(),
        ];
        let args = parse_args(args).unwrap();
        assert_eq!(args.e2e_yaml(), "example-e2e.yaml");
    }
}
