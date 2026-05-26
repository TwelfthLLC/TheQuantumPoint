//! Universal IR — target-agnostic; emitters (Rust, Go, …) consume this only.

mod expr;
mod program;

pub use expr::{emit_value_expr, sanitize_ident, BinOp, CmpOp, ValueExpr};
pub use program::{
    actions_need_async, Action, EnumDef, FunctionDef, Program, StructDef, SwitchArm,
};
