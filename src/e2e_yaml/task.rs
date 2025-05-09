use indexmap::{IndexMap, IndexSet};
use serde::Deserialize;

use super::step::Step;

#[derive(Debug, PartialEq)]
pub struct TaskError {
    pub kind: TaskErrorKind,
}

#[derive(Debug, PartialEq)]
pub enum TaskErrorKind {
    CircularDependenciesDetected(Vec<String>),
    TaskNotFound(String),
}

impl std::error::Error for TaskError {}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TaskErrorKind::CircularDependenciesDetected(path) => {
                writeln!(f, "circular dependencies detected. {}", path.join(" -> "))
            }
            TaskErrorKind::TaskNotFound(id) => {
                writeln!(f, "task with id '{}' not found  in configuration", id)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Tasks(pub IndexMap<String, Task>);

impl Tasks {
    pub fn detect_circular_dependencies(&self) -> Result<(), TaskError> {
        for key in self.0.keys() {
            let mut visited: IndexSet<&str> = IndexSet::new();
            self.detect_circular_dependencies_recursive(key, &mut visited)?;
        }
        Ok(())
    }

    fn detect_circular_dependencies_recursive<'a>(
        &'a self,
        cur_id: &'a str,
        visited: &mut IndexSet<&'a str>,
    ) -> Result<(), TaskError> {
        let task = match self.0.get(cur_id) {
            Some(task) => task,
            None => {
                return Err(TaskError {
                    kind: TaskErrorKind::TaskNotFound(cur_id.to_string()),
                });
            }
        };
        if !visited.insert(cur_id) {
            let mut path: Vec<String> = visited.iter().map(|x| x.to_string()).collect();
            path.push(cur_id.to_string());
            return Err(TaskError {
                kind: TaskErrorKind::CircularDependenciesDetected(path),
            });
        }
        if let Some(deps) = task.list_dependencies() {
            for dep_id in deps {
                self.detect_circular_dependencies_recursive(dep_id, visited)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Task {
    pub arg_names: Option<Vec<String>>,
    pub steps: Vec<Step>,
}

impl Task {
    pub fn list_dependencies(&self) -> Option<Vec<&str>> {
        let mut deps: Vec<&str> = Vec::new();
        for step in &self.steps {
            match step {
                Step::TaskRun { id, args: _ } => deps.push(id),
                _ => continue,
            }
        }
        if deps.is_empty() {
            None
        } else {
            Some(deps)
        }
    }

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

#[cfg(test)]
mod task_tests {
    use super::*;

    #[test]
    fn test_detect_mutual_dependencies() {
        let yaml = "
x:
  steps:
    - !task_run { id: y }

y:
  steps:
    - !task_run { id: x }
";
        let tasks: Tasks = serde_yaml::from_str(yaml).unwrap();
        let res = tasks.detect_circular_dependencies().err();
        assert_eq!(
            Some(TaskError {
                kind: TaskErrorKind::CircularDependenciesDetected(vec![
                    "x".to_string(),
                    "y".to_string(),
                    "x".to_string(),
                ])
            }),
            res
        );
    }

    #[test]
    fn test_detect_cycle_dependencies() {
        let yaml = "
t1:
  steps:
    - !task_run { id: t2 }

t2:
  steps:
    - !task_run { id: t3 }

t3:
  steps:
    - !task_run { id: t1 }
";
        let tasks: Tasks = serde_yaml::from_str(yaml).unwrap();
        let res = tasks.detect_circular_dependencies().err();
        assert_eq!(
            Some(TaskError {
                kind: TaskErrorKind::CircularDependenciesDetected(vec![
                    "t1".to_string(),
                    "t2".to_string(),
                    "t3".to_string(),
                    "t1".to_string(),
                ])
            }),
            res
        );
    }

    #[test]
    fn test_detect_cycle_dependencies2() {
        let yaml = "
t1:
  steps:
    - !task_run { id: t2 }

t2:
  steps:
    - !task_run { id: t1 }

t3:
  steps:
    - !task_run { id: t1 }
";
        let tasks: Tasks = serde_yaml::from_str(yaml).unwrap();
        let res = tasks.detect_circular_dependencies().err();
        assert_eq!(
            Some(TaskError {
                kind: TaskErrorKind::CircularDependenciesDetected(vec![
                    "t1".to_string(),
                    "t2".to_string(),
                    "t1".to_string(),
                ])
            }),
            res
        );
    }

    #[test]
    fn test_no_circular_dependencies() {
        let yaml = "
t1:
  steps:
    - !task_run { id: t2 }
t2:
  steps:
    - !goto localhost
";
        let tasks: Tasks = serde_yaml::from_str(yaml).unwrap();
        let res = tasks.detect_circular_dependencies();
        assert!(res.is_ok());
    }

    #[test]
    fn test_task_not_found() {
        let yaml = "
t1:
  steps:
    - !task_run { id: t2 }
";
        let tasks: Tasks = serde_yaml::from_str(yaml).unwrap();
        let res = tasks.detect_circular_dependencies().err();
        assert_eq!(
            Some(TaskError {
                kind: TaskErrorKind::TaskNotFound("t2".to_string())
            }),
            res
        );
    }

    #[test]
    fn test_self_cycle() {
        let yaml = "
t1:
  steps:
    - !task_run { id: t1 }
";
        let tasks: Tasks = serde_yaml::from_str(yaml).unwrap();
        let res = tasks.detect_circular_dependencies().err();
        let expected_path = vec!["t1".to_string(), "t1".to_string()];
        assert_eq!(
            Some(TaskError {
                kind: TaskErrorKind::CircularDependenciesDetected(expected_path)
            }),
            res
        );
    }
}
