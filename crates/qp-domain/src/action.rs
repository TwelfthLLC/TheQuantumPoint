use serde::{Deserialize, Serialize};

/// Language-agnostic domain action (lowered to universal IR, then to Rust/Go/etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DomainAction {
    Print {
        message: String,
    },
    DataStore {
        name: String,
        value: ActionValue,
    },
    Const {
        name: String,
        value: ActionValue,
    },
    ListStore {
        name: String,
        items: Vec<ActionValue>,
    },
    Branch {
        condition: ActionValue,
        then_body: Vec<DomainAction>,
        else_body: Vec<DomainAction>,
    },
    While {
        condition: ActionValue,
        body: Vec<DomainAction>,
    },
    For {
        var: String,
        from: i64,
        to: i64,
        body: Vec<DomainAction>,
    },
    ForEach {
        item_var: String,
        collection: String,
        body: Vec<DomainAction>,
    },
    Return {
        value: Option<ActionValue>,
    },
    Switch {
        discriminant: ActionValue,
        arms: Vec<SwitchArm>,
        default_body: Vec<DomainAction>,
    },
    Break,
    Continue,
    Try {
        try_body: Vec<DomainAction>,
        catch_body: Vec<DomainAction>,
    },
    Throw {
        message: String,
    },
    Expr {
        name: String,
        value: ActionValue,
    },
    Async {
        body: Vec<DomainAction>,
    },
    Await {
        binding: Option<String>,
    },
    Call {
        name: String,
        args: Vec<ActionValue>,
        into: Option<String>,
    },
    DbRead {
        table: String,
        into_var: String,
    },
    Module {
        name: String,
        actions: Vec<DomainAction>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchArm {
    pub label: String,
    pub body: Vec<DomainAction>,
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
        op: ArithOp,
        left: Box<ActionValue>,
        right: Box<ActionValue>,
    },
    Logic {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}
