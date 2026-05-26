use ir::{BinOp, CmpOp, ValueExpr};
use std::collections::HashMap;

use super::RuntimeError;

#[derive(Clone)]
pub(crate) enum RuntimeValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
    List(Vec<RuntimeValue>),
}

impl RuntimeValue {
    pub(crate) fn as_bool(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::I64(n) => *n != 0,
            RuntimeValue::F64(n) => *n != 0.0,
            RuntimeValue::Str(s) => !s.is_empty(),
            RuntimeValue::List(v) => !v.is_empty(),
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
            RuntimeValue::List(v) => write!(f, "[{v:?}]"),
        }
    }
}

pub(crate) fn runtime_to_switch_key(v: &RuntimeValue) -> String {
    match v {
        RuntimeValue::I64(n) => n.to_string(),
        RuntimeValue::Bool(b) => b.to_string(),
        RuntimeValue::Str(s) => s.clone(),
        RuntimeValue::F64(n) => n.to_string(),
        RuntimeValue::List(_) => "list".into(),
    }
}

pub(crate) fn eval_value(
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
