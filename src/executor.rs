use crate::markdown::CodeBlockBody;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::process::{Command, Output, Stdio};

/// Execute code blocks for a particular language.
pub struct LangExecutor {
    program: String,
    base_args: Vec<String>,
}

impl LangExecutor {
    pub fn new(program: String, base_args: Vec<String>) -> Self {
        LangExecutor { program, base_args }
    }

    pub fn execute_body(&self, body: &CodeBlockBody) -> Result<()> {
        let _ = Command::new(self.program.clone())
            .args(self.base_args.clone())
            .arg(body.code.clone())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

pub struct Executors(HashMap<String, LangExecutor>);

impl Executors {
    pub fn default_executors() -> Self {
        let mut exs = HashMap::new();
        exs.insert(
            "sh".to_owned(),
            LangExecutor::new("sh".to_owned(), vec!["-c".to_owned()]),
        );
        exs.insert(
            "bash".to_owned(),
            LangExecutor::new("bash".to_owned(), vec!["-c".to_owned()]),
        );
        exs.insert(
            "shell".to_owned(),
            LangExecutor::new("sh".to_owned(), vec!["-c".to_owned()]),
        );
        Executors(exs)
    }

    pub fn execute(&self, body: &CodeBlockBody) -> Result<()> {
        match &body.lang {
            Some(lang) => {
                if let Some(ex) = self.0.get(lang) {
                    ex.execute_body(body)?;
                }
                Ok(())
            }
            None => Ok(()),
        }
    }
}
