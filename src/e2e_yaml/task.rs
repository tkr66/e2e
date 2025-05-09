use indexmap::IndexMap;
use serde::Deserialize;

use super::step::Step;

#[derive(Debug, Deserialize)]
pub struct Tasks(pub IndexMap<String, Task>);

#[derive(Debug, PartialEq, Deserialize)]
pub struct Task {
    pub arg_names: Option<Vec<String>>,
    pub steps: Vec<Step>,
}

impl Task {
    pub fn expand_args(&self, args: Option<&[&str]>) -> Vec<Step> {
        let mut result: Vec<Step> = Vec::new();
        for step in &self.steps {
            let mut cloned = step.clone();
            if let Some(names) = &self.arg_names {
                (0..names.len()).for_each(|i| {
                    cloned = cloned.expand_var(names[i].as_str(), args.unwrap()[i]);
                });
            }
            result.push(cloned);
        }
        result
    }
}
