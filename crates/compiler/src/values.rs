use crate::{safe_expr, CompileError};
use ir::{BinOp, CmpOp, ValueExpr};
use qp_domain::{ActionValue, ArithOp, LogicOp};

pub(crate) fn action_value_from_condition(s: &str) -> Result<ActionValue, CompileError> {
    let expr = safe_expr::parse_condition(s)?;
    Ok(ir_expr_to_action_value(expr))
}

fn ir_expr_to_action_value(expr: ValueExpr) -> ActionValue {
    match expr {
        ValueExpr::Bool(b) => ActionValue::Bool(b),
        ValueExpr::I64(n) => ActionValue::I64(n),
        ValueExpr::F64(n) => ActionValue::F64(n),
        ValueExpr::Str(s) => ActionValue::Str(s),
        ValueExpr::Ident(name) => ActionValue::Ident(name),
        ValueExpr::Cmp { op, left, right } => ActionValue::Cmp {
            op: match op {
                CmpOp::Eq => qp_domain::CmpOp::Eq,
                CmpOp::Ne => qp_domain::CmpOp::Ne,
                CmpOp::Lt => qp_domain::CmpOp::Lt,
                CmpOp::Le => qp_domain::CmpOp::Le,
                CmpOp::Gt => qp_domain::CmpOp::Gt,
                CmpOp::Ge => qp_domain::CmpOp::Ge,
            },
            left: Box::new(ir_expr_to_action_value(*left)),
            right: Box::new(ir_expr_to_action_value(*right)),
        },
        ValueExpr::BinOp { op, left, right } => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => ActionValue::BinOp {
                op: match op {
                    BinOp::Add => ArithOp::Add,
                    BinOp::Sub => ArithOp::Sub,
                    BinOp::Mul => ArithOp::Mul,
                    BinOp::Div => ArithOp::Div,
                    _ => ArithOp::Add,
                },
                left: Box::new(ir_expr_to_action_value(*left)),
                right: Box::new(ir_expr_to_action_value(*right)),
            },
            BinOp::And | BinOp::Or => ActionValue::Logic {
                op: match op {
                    BinOp::And => LogicOp::And,
                    BinOp::Or => LogicOp::Or,
                    _ => LogicOp::And,
                },
                left: Box::new(ir_expr_to_action_value(*left)),
                right: Box::new(ir_expr_to_action_value(*right)),
            },
        },
        ValueExpr::Not(inner) => ActionValue::Not(Box::new(ir_expr_to_action_value(*inner))),
    }
}
