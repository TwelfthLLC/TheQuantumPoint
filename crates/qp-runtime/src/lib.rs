//! Safe in-process interpreter for `ir::Program` (Run preview).

mod exec;
mod flow;
mod mock;
mod value;

#[cfg(test)]
mod tests;

use ir::Program;
use std::collections::HashMap;
use thiserror::Error;

use exec::run_actions;
use flow::Flow;
use value::RuntimeValue;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("unknown variable '{0}'")]
    UnknownVar(String),
    #[error("unknown function '{0}'")]
    UnknownFn(String),
    #[error("break outside loop")]
    BreakOutsideLoop,
    #[error("continue outside loop")]
    ContinueOutsideLoop,
    #[error("runtime: {0}")]
    Message(String),
}

#[derive(Debug, Clone)]
pub struct RunPreview {
    pub lines: Vec<String>,
}

/// Execute IR actions (control flow, print, assign, loops, switch, try stub).
pub fn interpret(program: &Program) -> Result<RunPreview, RuntimeError> {
    let functions: HashMap<&str, &ir::FunctionDef> = program
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();
    let mut env: HashMap<String, RuntimeValue> = HashMap::new();
    let mut lines = Vec::new();
    match run_actions(&program.actions, &mut env, &mut lines, false, &functions) {
        Ok(Flow::Next) | Ok(Flow::Return(_)) => Ok(RunPreview { lines }),
        Ok(Flow::Break) => Err(RuntimeError::BreakOutsideLoop),
        Ok(Flow::Continue) => Err(RuntimeError::ContinueOutsideLoop),
        Err(e) => Err(e),
    }
}
