use super::step::Step;
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub struct ScenarioError {
    kind: ScenarioErrorKind,
}

impl std::error::Error for ScenarioError {}

impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ScenarioErrorKind::NotFound => write!(f, "NotFound"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ScenarioErrorKind {
    NotFound,
}

#[derive(Debug, Deserialize)]
pub struct Scenarios(pub IndexMap<String, Scenario>);

impl Scenarios {
    pub fn find(&self, id: &[&str]) -> Result<Vec<&Scenario>, ScenarioError> {
        let mut result: Vec<&Scenario> = Vec::new();
        for ele in id {
            if let Some(v) = self.0.get(*ele) {
                result.push(v);
            } else {
                return Err(ScenarioError {
                    kind: ScenarioErrorKind::NotFound,
                });
            }
        }
        Ok(result)
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
}

#[cfg(test)]
mod scenario_test {
    use super::*;

    #[test]
    fn test_find() {
        let mut map: IndexMap<String, Scenario> = IndexMap::new();
        let data: Vec<(&str, &str)> = vec![
            ("id1", "name1"),
            ("id2", "name2"),
            ("id3", "name3"),
            ("id4", "name4"),
            ("id5", "name5"),
        ];
        for (id, name) in data {
            map.insert(
                id.to_string(),
                Scenario {
                    name: name.to_string(),
                    steps: Vec::new(),
                },
            );
        }
        let scenarios = Scenarios(map);
        let id = vec!["id1", "id5"];

        let expected = [
            Scenario {
                name: "name1".to_string(),
                steps: Vec::new(),
            },
            Scenario {
                name: "name5".to_string(),
                steps: Vec::new(),
            },
        ];
        let actual = scenarios.find(id.as_slice()).unwrap();
        assert_eq!(2, actual.len());
        assert_eq!(&expected[0], actual[0]);
        assert_eq!(&expected[1], actual[1]);
    }

    #[test]
    fn test_find_not_found() {
        let mut map: IndexMap<String, Scenario> = IndexMap::new();
        let data: Vec<(&str, &str)> = vec![
            ("id1", "name1"),
            ("id2", "name2"),
            ("id3", "name3"),
            ("id4", "name4"),
            ("id5", "name5"),
        ];
        for (id, name) in data {
            map.insert(
                id.to_string(),
                Scenario {
                    name: name.to_string(),
                    steps: Vec::new(),
                },
            );
        }
        let scenarios = Scenarios(map);
        let id = vec!["id6"];

        let actual = scenarios.find(id.as_slice());
        let err = actual.err();
        assert!(err.is_some());
        assert_eq!(
            ScenarioError {
                kind: ScenarioErrorKind::NotFound
            },
            err.unwrap()
        );
    }
}
