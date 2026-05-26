//! Safe in-process interpreter for `ir::Program` (Run preview).

use ir::{Action, BinOp, CmpOp, Program, ValueExpr};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("unknown variable '{0}'")]
    UnknownVar(String),
    #[error("runtime: {0}")]
    Message(String),
}

#[derive(Debug, Clone)]
pub struct RunPreview {
    pub lines: Vec<String>,
}

/// Execute IR actions (Print, Assign, Branch, DbRead mock only).
pub fn interpret(program: &Program) -> Result<RunPreview, RuntimeError> {
    let mut env: HashMap<String, RuntimeValue> = HashMap::new();
    let mut lines = Vec::new();
    run_actions(&program.actions, &mut env, &mut lines)?;
    Ok(RunPreview { lines })
}

fn run_actions(
    actions: &[Action],
    env: &mut HashMap<String, RuntimeValue>,
    lines: &mut Vec<String>,
) -> Result<(), RuntimeError> {
    for action in actions {
        run_action(action, env, lines)?;
    }
    Ok(())
}

fn run_action(
    action: &Action,
    env: &mut HashMap<String, RuntimeValue>,
    lines: &mut Vec<String>,
) -> Result<(), RuntimeError> {
    match action {
        Action::Print { message } => {
            lines.push(message.clone());
        }
        Action::DataStore { name, value } => {
            let v = eval_value(value, env)?;
            env.insert(name.clone(), v);
        }
        Action::Branch {
            condition,
            then_body,
            else_body,
        } => {
            if eval_value(condition, env)?.as_bool() {
                run_actions(then_body, env, lines)?;
            } else {
                run_actions(else_body, env, lines)?;
            }
        }
        Action::DbRead { table, into_var } => {
            let row = RuntimeValue::Str(format!("mock-row-from-{table}"));
            env.insert(into_var.clone(), row.clone());
            lines.push(format!("db.read {table} -> {row:?}"));
        }
        Action::Module { name, actions } => {
            lines.push(format!("— module {name} —"));
            run_actions(actions, env, lines)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
enum RuntimeValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
}

impl RuntimeValue {
    fn as_bool(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::I64(n) => *n != 0,
            RuntimeValue::F64(n) => *n != 0.0,
            RuntimeValue::Str(s) => !s.is_empty(),
        }
    }
}

fn eval_value(
    expr: &ValueExpr,
    env: &HashMap<String, RuntimeValue>,
) -> Result<RuntimeValue, RuntimeError> {
    match expr {
        ValueExpr::Bool(b) => Ok(RuntimeValue::Bool(*b)),
        ValueExpr::I64(n) => Ok(RuntimeValue::I64(*n)),
        ValueExpr::F64(n) => Ok(RuntimeValue::F64(*n)),
        ValueExpr::Str(s) => Ok(RuntimeValue::Str(s.clone())),
        ValueExpr::Ident(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::UnknownVar(name.clone())),
        ValueExpr::Not(inner) => Ok(RuntimeValue::Bool(!eval_value(inner, env)?.as_bool())),
        ValueExpr::Cmp { op, left, right } => {
            let l = eval_value(left, env)?;
            let r = eval_value(right, env)?;
            Ok(RuntimeValue::Bool(cmp_values(*op, &l, &r)))
        }
        ValueExpr::BinOp { op, left, right } => {
            let l = eval_value(left, env)?;
            let r = eval_value(right, env)?;
            bin_values(*op, &l, &r)
        }
    }
}

fn cmp_values(op: CmpOp, l: &RuntimeValue, r: &RuntimeValue) -> bool {
    match (l, r) {
        (RuntimeValue::I64(a), RuntimeValue::I64(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
        },
        (RuntimeValue::Str(a), RuntimeValue::Str(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            _ => false,
        },
        (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            _ => false,
        },
        _ => false,
    }
}

fn bin_values(op: BinOp, l: &RuntimeValue, r: &RuntimeValue) -> Result<RuntimeValue, RuntimeError> {
    match op {
        BinOp::And => Ok(RuntimeValue::Bool(l.as_bool() && r.as_bool())),
        BinOp::Or => Ok(RuntimeValue::Bool(l.as_bool() || r.as_bool())),
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
            let (a, b) = match (l, r) {
                (RuntimeValue::I64(a), RuntimeValue::I64(b)) => (*a, *b),
                _ => {
                    return Err(RuntimeError::Message(
                        "arithmetic only on i64 in preview".into(),
                    ))
                }
            };
            let n = match op {
                BinOp::Add => a + b,
                BinOp::Sub => a - b,
                BinOp::Mul => a * b,
                BinOp::Div => a / b,
                _ => a,
            };
            Ok(RuntimeValue::I64(n))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ir::Action;

    #[test]
    fn print_preview() {
        let p = Program {
            name: "t".into(),
            actions: vec![Action::Print {
                message: "hi".into(),
            }],
        };
        let out = interpret(&p).unwrap();
        assert_eq!(out.lines, vec!["hi"]);
    }
}
