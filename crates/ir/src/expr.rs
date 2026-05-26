use serde::{Deserialize, Serialize};

/// Safe, structured values — no embedded target-language source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueExpr {
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
    Ident(String),
    Cmp {
        op: CmpOp,
        left: Box<ValueExpr>,
        right: Box<ValueExpr>,
    },
    BinOp {
        op: BinOp,
        left: Box<ValueExpr>,
        right: Box<ValueExpr>,
    },
    Not(Box<ValueExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
}

pub fn sanitize_ident(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if i == 0 {
            if ch.is_ascii_alphabetic() || ch == '_' {
                out.push(ch);
            } else {
                out.push('_');
                if ch.is_ascii_alphanumeric() {
                    out.push(ch);
                }
            }
        } else if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

pub fn emit_value_expr(expr: &ValueExpr) -> String {
    match expr {
        ValueExpr::Bool(b) => b.to_string(),
        ValueExpr::I64(n) => n.to_string(),
        ValueExpr::F64(n) => format!("{n}"),
        ValueExpr::Str(s) => format!("{s:?}"),
        ValueExpr::Ident(name) => sanitize_ident(name),
        ValueExpr::Not(inner) => format!("!({})", emit_value_expr(inner)),
        ValueExpr::Cmp { op, left, right } => {
            let op_s = match op {
                CmpOp::Eq => "==",
                CmpOp::Ne => "!=",
                CmpOp::Lt => "<",
                CmpOp::Le => "<=",
                CmpOp::Gt => ">",
                CmpOp::Ge => ">=",
            };
            format!(
                "({} {} {})",
                emit_value_expr(left),
                op_s,
                emit_value_expr(right)
            )
        }
        ValueExpr::BinOp { op, left, right } => {
            let op_s = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::And => "&&",
                BinOp::Or => "||",
            };
            format!(
                "({} {} {})",
                emit_value_expr(left),
                op_s,
                emit_value_expr(right)
            )
        }
    }
}
