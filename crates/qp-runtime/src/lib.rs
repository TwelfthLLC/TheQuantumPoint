//! Safe in-process interpreter for `ir::Program` (Run preview).

use ir::{Action, BinOp, CmpOp, Program, ValueExpr};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("unknown variable '{0}'")]
    UnknownVar(String),
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

enum Flow {
    Next,
    Return,
    Break,
    Continue,
}

/// Execute IR actions (control flow, print, assign, loops, switch, try stub).
pub fn interpret(program: &Program) -> Result<RunPreview, RuntimeError> {
    let mut env: HashMap<String, RuntimeValue> = HashMap::new();
    let mut lines = Vec::new();
    match run_actions(&program.actions, &mut env, &mut lines, false) {
        Ok(Flow::Next) | Ok(Flow::Return) => Ok(RunPreview { lines }),
        Ok(Flow::Break) => Err(RuntimeError::BreakOutsideLoop),
        Ok(Flow::Continue) => Err(RuntimeError::ContinueOutsideLoop),
        Err(e) => Err(e),
    }
}

fn run_actions(
    actions: &[Action],
    env: &mut HashMap<String, RuntimeValue>,
    lines: &mut Vec<String>,
    in_loop: bool,
) -> Result<Flow, RuntimeError> {
    for action in actions {
        match run_action(action, env, lines, in_loop)? {
            Flow::Next => {}
            other => return Ok(other),
        }
    }
    Ok(Flow::Next)
}

fn run_action(
    action: &Action,
    env: &mut HashMap<String, RuntimeValue>,
    lines: &mut Vec<String>,
    in_loop: bool,
) -> Result<Flow, RuntimeError> {
    match action {
        Action::Print { message } => {
            lines.push(message.clone());
            Ok(Flow::Next)
        }
        Action::DataStore { name, value } => {
            let v = eval_value(value, env)?;
            env.insert(name.clone(), v);
            Ok(Flow::Next)
        }
        Action::Branch {
            condition,
            then_body,
            else_body,
        } => {
            if eval_value(condition, env)?.as_bool() {
                run_actions(then_body, env, lines, in_loop)
            } else {
                run_actions(else_body, env, lines, in_loop)
            }
        }
        Action::While { condition, body } => {
            while eval_value(condition, env)?.as_bool() {
                match run_actions(body, env, lines, true)? {
                    Flow::Next => {}
                    Flow::Break => break,
                    Flow::Continue => continue,
                    other => return Ok(other),
                }
            }
            Ok(Flow::Next)
        }
        Action::ForEach {
            item_var,
            collection,
            body,
        } => {
            for row in mock_collection(collection) {
                env.insert(item_var.clone(), row);
                match run_actions(body, env, lines, true)? {
                    Flow::Next => {}
                    Flow::Break => break,
                    Flow::Continue => continue,
                    other => return Ok(other),
                }
            }
            Ok(Flow::Next)
        }
        Action::For { var, from, to, body } => {
            let from = *from;
            let to = *to;
            let step = if from <= to { 1 } else { -1 };
            let mut i = from;
            loop {
                if step > 0 && i > to {
                    break;
                }
                if step < 0 && i < to {
                    break;
                }
                env.insert(var.clone(), RuntimeValue::I64(i));
                match run_actions(body, env, lines, true)? {
                    Flow::Next => {}
                    Flow::Break => break,
                    Flow::Continue => {
                        i += step;
                        continue;
                    }
                    other => return Ok(other),
                }
                i += step;
            }
            Ok(Flow::Next)
        }
        Action::Return { value } => {
            if let Some(v) = value {
                let rv = eval_value(v, env)?;
                lines.push(format!("return {rv:?}"));
            } else {
                lines.push("return".into());
            }
            Ok(Flow::Return)
        }
        Action::Switch {
            discriminant,
            arms,
            default_body,
        } => {
            let key = runtime_to_switch_key(&eval_value(discriminant, env)?);
            for arm in arms {
                if arm.label == key {
                    return run_actions(&arm.body, env, lines, in_loop);
                }
            }
            run_actions(default_body, env, lines, in_loop)
        }
        Action::Break => {
            if in_loop {
                Ok(Flow::Break)
            } else {
                Err(RuntimeError::BreakOutsideLoop)
            }
        }
        Action::Continue => {
            if in_loop {
                Ok(Flow::Continue)
            } else {
                Err(RuntimeError::ContinueOutsideLoop)
            }
        }
        Action::Try {
            try_body,
            catch_body,
        } => {
            lines.push("try {".into());
            let try_flow = run_actions(try_body, env, lines, in_loop);
            if matches!(try_flow, Err(RuntimeError::Message(_)) | Err(RuntimeError::UnknownVar(_))) {
                lines.push("} catch {".into());
                run_actions(catch_body, env, lines, in_loop)
            } else {
                lines.push("} // try ok".into());
                try_flow
            }
        }
        Action::Expr { name, value } => {
            let v = eval_value(value, env)?;
            env.insert(name.clone(), v);
            Ok(Flow::Next)
        }
        Action::Async { body } => {
            lines.push("async { ... }".into());
            run_actions(body, env, lines, in_loop)
        }
        Action::DbRead { table, into_var } => {
            let row = mock_collection(table)
                .into_iter()
                .next()
                .unwrap_or(RuntimeValue::Str("empty".into()));
            env.insert(into_var.clone(), row.clone());
            lines.push(format!("db.read {table} -> {row:?}"));
            Ok(Flow::Next)
        }
        Action::Module { name, actions } => {
            lines.push(format!("— module {name} —"));
            run_actions(actions, env, lines, in_loop)
        }
    }
}

fn mock_collection(table: &str) -> Vec<RuntimeValue> {
    match table {
        "users" => vec![
            RuntimeValue::Str("user:1:Ada".into()),
            RuntimeValue::Str("user:2:Bob".into()),
        ],
        "orders" => vec![
            RuntimeValue::Str("order:100".into()),
            RuntimeValue::Str("order:101".into()),
        ],
        other => vec![RuntimeValue::Str(format!("row-from-{other}"))],
    }
}

fn runtime_to_switch_key(v: &RuntimeValue) -> String {
    match v {
        RuntimeValue::I64(n) => n.to_string(),
        RuntimeValue::Bool(b) => b.to_string(),
        RuntimeValue::Str(s) => s.clone(),
        RuntimeValue::F64(n) => n.to_string(),
    }
}

#[derive(Clone)]
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

impl std::fmt::Debug for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeValue::Bool(b) => write!(f, "{b}"),
            RuntimeValue::I64(n) => write!(f, "{n}"),
            RuntimeValue::F64(n) => write!(f, "{n}"),
            RuntimeValue::Str(s) => write!(f, "{s:?}"),
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
            needs_async_runtime: false,
            actions: vec![Action::Print {
                message: "hi".into(),
            }],
        };
        let out = interpret(&p).unwrap();
        assert_eq!(out.lines, vec!["hi"]);
    }
}
