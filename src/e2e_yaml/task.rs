use indexmap::IndexMap;
use serde::Deserialize;
use thirtyfour::{error::WebDriverError, WebDriver};

use super::{step::Step, E2eYaml};

#[derive(Debug, Deserialize)]
pub struct Tasks(pub IndexMap<String, Task>);

#[derive(Debug, PartialEq, Deserialize)]
pub struct Task {
    pub arg_names: Option<Vec<String>>,
    pub steps: Vec<Step>,
}

impl Task {
    pub async fn run(
        &self,
        driver: &WebDriver,
        config: &E2eYaml,
        args: Option<&Vec<String>>,
    ) -> Result<(), WebDriverError> {
        let mut steps: Vec<Step> = Vec::new();
        for s in self.steps.iter() {
            let mut step = s.expand_vars(&config.vars);
            if let Some(args) = args {
                for i in 0..self.arg_names.as_ref().unwrap().len() {
                    let arg_name = self.arg_names.as_ref().unwrap().get(i).unwrap();
                    let arg = args.get(i).unwrap();
                    step = step.expand_var(arg_name.as_str(), arg.as_str());
                }
            }
            steps.push(step);
        }
        for s in steps {
            Box::pin(s.run(driver, config)).await?;
        }

        Ok(())
    }
}
