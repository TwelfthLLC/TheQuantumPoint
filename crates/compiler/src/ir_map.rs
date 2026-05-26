use ir::{Action, BinOp, CmpOp, SwitchArm, ValueExpr};
use qp_domain::{ActionValue, ArithOp, DomainAction, LogicOp};

pub(crate) fn domain_action_to_ir(action: DomainAction) -> Action {
    match action {
        DomainAction::Print { message } => Action::Print { message },
        DomainAction::DataStore { name, value } => Action::DataStore {
            name,
            value: value_to_ir(value),
        },
        DomainAction::Branch {
            condition,
            then_body,
            else_body,
        } => Action::Branch {
            condition: value_to_ir(condition),
            then_body: then_body.into_iter().map(domain_action_to_ir).collect(),
            else_body: else_body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::DbRead { table, into_var } => Action::DbRead { table, into_var },
        DomainAction::While { condition, body } => Action::While {
            condition: value_to_ir(condition),
            body: body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::For {
            var,
            from,
            to,
            body,
        } => Action::For {
            var,
            from,
            to,
            body: body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::ForEach {
            item_var,
            collection,
            body,
        } => Action::ForEach {
            item_var,
            collection,
            body: body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::Return { value } => Action::Return {
            value: value.map(value_to_ir),
        },
        DomainAction::Switch {
            discriminant,
            arms,
            default_body,
        } => Action::Switch {
            discriminant: value_to_ir(discriminant),
            arms: arms
                .into_iter()
                .map(|a| SwitchArm {
                    label: a.label,
                    body: a.body.into_iter().map(domain_action_to_ir).collect(),
                })
                .collect(),
            default_body: default_body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::Break => Action::Break,
        DomainAction::Continue => Action::Continue,
        DomainAction::Try {
            try_body,
            catch_body,
        } => Action::Try {
            try_body: try_body.into_iter().map(domain_action_to_ir).collect(),
            catch_body: catch_body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::Expr { name, value } => Action::Expr {
            name,
            value: value_to_ir(value),
        },
        DomainAction::Async { body } => Action::Async {
            body: body.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::Module { name, actions } => Action::Module {
            name,
            actions: actions.into_iter().map(domain_action_to_ir).collect(),
        },
        DomainAction::Const { name, value } => Action::Const {
            name,
            value: value_to_ir(value),
        },
        DomainAction::ListStore { name, items } => Action::ListStore {
            name,
            items: items.into_iter().map(value_to_ir).collect(),
        },
        DomainAction::Throw { message } => Action::Throw { message },
        DomainAction::Await { binding } => Action::Await { binding },
        DomainAction::Call { name, args, into } => Action::Call {
            name,
            args: args.into_iter().map(value_to_ir).collect(),
            into,
        },
    }
}

fn value_to_ir(v: ActionValue) -> ValueExpr {
    match v {
        ActionValue::Bool(b) => ValueExpr::Bool(b),
        ActionValue::I64(n) => ValueExpr::I64(n),
        ActionValue::F64(n) => ValueExpr::F64(n),
        ActionValue::Str(s) => ValueExpr::Str(s),
        ActionValue::Ident(name) => ValueExpr::Ident(name),
        ActionValue::Cmp { op, left, right } => ValueExpr::Cmp {
            op: match op {
                qp_domain::CmpOp::Eq => CmpOp::Eq,
                qp_domain::CmpOp::Ne => CmpOp::Ne,
                qp_domain::CmpOp::Lt => CmpOp::Lt,
                qp_domain::CmpOp::Le => CmpOp::Le,
                qp_domain::CmpOp::Gt => CmpOp::Gt,
                qp_domain::CmpOp::Ge => CmpOp::Ge,
            },
            left: Box::new(value_to_ir(*left)),
            right: Box::new(value_to_ir(*right)),
        },
        ActionValue::BinOp { op, left, right } => ValueExpr::BinOp {
            op: match op {
                ArithOp::Add => BinOp::Add,
                ArithOp::Sub => BinOp::Sub,
                ArithOp::Mul => BinOp::Mul,
                ArithOp::Div => BinOp::Div,
            },
            left: Box::new(value_to_ir(*left)),
            right: Box::new(value_to_ir(*right)),
        },
        ActionValue::Logic { op, left, right } => ValueExpr::BinOp {
            op: match op {
                LogicOp::And => BinOp::And,
                LogicOp::Or => BinOp::Or,
            },
            left: Box::new(value_to_ir(*left)),
            right: Box::new(value_to_ir(*right)),
        },
        ActionValue::Not(inner) => ValueExpr::Not(Box::new(value_to_ir(*inner))),
    }
}
