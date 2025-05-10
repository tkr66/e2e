use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct Vars(pub IndexMap<String, String>);

pub fn parse_var_names(input: &str) -> Option<Vec<String>> {
    let mut buf: Option<String> = None;
    let mut result: Vec<String> = Vec::new();
    let mut peekable = input.chars().peekable();
    while let Some(c) = peekable.next() {
        match c {
            '{' => {
                // escape
                if let Some(&'{') = peekable.peek() {
                    peekable.next();
                    continue;
                }
                buf = Some(String::new());
            }
            '}' => {
                // escape
                if let Some(&'}') = peekable.peek() {
                    peekable.next();
                    continue;
                }
                if buf.is_some() {
                    result.push(buf.clone().unwrap());
                }
            }
            _ => {
                if let Some(ref mut b) = buf {
                    b.push(c);
                }
            }
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod var_tests {
    use super::*;

    #[test]
    fn test_single_variable() {
        let input = "{name}";
        let expected = Some(vec!["name".to_string()]);
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_multiple_variables() {
        let input = "{a} {b} {c} {{{d}}}";
        let expected = Some(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]);
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_escaped_braces() {
        let input = "{{not_a_var}} and {real}";
        let expected = Some(vec!["real".to_string()]);
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let expected = None;
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_no_variables() {
        let input = "hello world";
        let expected = None;
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_trailing_brace_without_opening() {
        let input = "oops}";
        let expected = None;
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_escape_sequence_only() {
        let input = "{{}}";
        let expected = None;
        assert_eq!(parse_var_names(input), expected);
    }

    #[test]
    fn test_whitespace_inside_var() {
        let input = "{hello world}";
        let expected = Some(vec!["hello world".to_string()]);
        assert_eq!(parse_var_names(input), expected);
    }
}
