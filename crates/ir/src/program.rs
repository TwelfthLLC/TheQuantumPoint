use serde::{Deserialize, Serialize};

use crate::ValueExpr;

/// Universal intermediate representation (flattened control flow).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Print {
        message: String,
    },
    DataStore {
        name: String,
        value: ValueExpr,
    },
    Branch {
        condition: ValueExpr,
        then_body: Vec<Action>,
        else_body: Vec<Action>,
    },
    DbRead {
        table: String,
        into_var: String,
    },
    Module {
        name: String,
        actions: Vec<Action>,
    },
}
