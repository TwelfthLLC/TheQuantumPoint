use serde::{Deserialize, Serialize};

use crate::ValueExpr;

/// Universal intermediate representation (flattened control flow).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    pub actions: Vec<Action>,
    /// Set when the graph contains `async` blocks (emit adds tokio).
    #[serde(default)]
    pub needs_async_runtime: bool,
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
    While {
        condition: ValueExpr,
        body: Vec<Action>,
    },
    For {
        var: String,
        from: i64,
        to: i64,
        body: Vec<Action>,
    },
    ForEach {
        item_var: String,
        collection: String,
        body: Vec<Action>,
    },
    Return {
        value: Option<ValueExpr>,
    },
    Switch {
        discriminant: ValueExpr,
        arms: Vec<SwitchArm>,
        default_body: Vec<Action>,
    },
    Break,
    Continue,
    Try {
        try_body: Vec<Action>,
        catch_body: Vec<Action>,
    },
    Expr {
        name: String,
        value: ValueExpr,
    },
    Async {
        body: Vec<Action>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchArm {
    pub label: String,
    pub body: Vec<Action>,
}

/// True when any action tree contains `Action::Async`.
pub fn actions_need_async(actions: &[Action]) -> bool {
    actions.iter().any(action_needs_async)
}

fn action_needs_async(action: &Action) -> bool {
    match action {
        Action::Async { .. } => true,
        Action::Module { actions, .. } => actions_need_async(actions),
        Action::Branch {
            then_body,
            else_body,
            ..
        } => actions_need_async(then_body) || actions_need_async(else_body),
        Action::While { body, .. } | Action::For { body, .. } | Action::ForEach { body, .. } => {
            actions_need_async(body)
        }
        Action::Switch {
            arms,
            default_body,
            ..
        } => {
            arms.iter()
                .any(|a| actions_need_async(&a.body))
                || actions_need_async(default_body)
        }
        Action::Try {
            try_body,
            catch_body,
        } => actions_need_async(try_body) || actions_need_async(catch_body),
        _ => false,
    }
}
