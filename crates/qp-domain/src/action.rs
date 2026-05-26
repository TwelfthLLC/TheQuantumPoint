use serde::{Deserialize, Serialize};

/// Language-agnostic domain action (lowered to universal IR, then to Rust/Go/etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DomainAction {
    /// `Action::Print` — stdout / log sink.
    Print { message: String },
    /// `Action::DataStore` — bind a name to a value in the current scope.
    DataStore { name: String, value: ActionValue },
    /// `Action::Branch` — conditional exec split.
    Branch {
        condition: ActionValue,
        then_body: Vec<DomainAction>,
        else_body: Vec<DomainAction>,
    },
    /// Mock DB read — runtime uses in-memory row; emitters generate stubs.
    DbRead { table: String, into_var: String },
    /// Inline compiled subgraph module.
    Module {
        name: String,
        actions: Vec<DomainAction>,
    },
}

/// Typed literal / structured expressions (no target-language snippets).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
    Ident(String),
    Cmp {
        op: CmpOp,
        left: Box<ActionValue>,
        right: Box<ActionValue>,
    },
    BinOp {
        op: LogicOp,
        left: Box<ActionValue>,
        right: Box<ActionValue>,
    },
    Not(Box<ActionValue>),
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
pub enum LogicOp {
    And,
    Or,
}
