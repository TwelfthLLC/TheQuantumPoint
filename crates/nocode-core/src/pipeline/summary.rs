use ir::{Action, Program};

pub(crate) fn format_program_summary(program: &Program) -> String {
    let mut lines = vec![
        "✓ Core domain → universal IR (tilsiz)".to_string(),
        format!("• dastur: {}", program.name),
        format!("• amallar: {}", program.actions.len()),
    ];
    for (i, action) in program.actions.iter().enumerate() {
        lines.push(format!("  [{i}] {}", action_summary(action)));
    }
    lines.push("Build → Rust (yoki boshqa emit) + cargo.".to_string());
    lines.join("\n")
}

fn action_summary(action: &Action) -> String {
    match action {
        Action::Print { message } => format!("print {message:?}"),
        Action::DataStore { name, value } => format!("let {name} = {value:?}"),
        Action::Branch {
            condition,
            then_body,
            else_body,
        } => format!(
            "if {condition:?} then {} else {}",
            then_body.len(),
            else_body.len()
        ),
        Action::DbRead { table, into_var } => format!("db.read {table} → {into_var}"),
        Action::While { condition, body } => {
            format!("while {condition:?} body {} actions", body.len())
        }
        Action::For {
            var,
            from,
            to,
            body,
        } => {
            format!("for {var} in {from}..={to} body {} actions", body.len())
        }
        Action::ForEach {
            item_var,
            collection,
            body,
        } => format!(
            "foreach {item_var} in {collection} body {} actions",
            body.len()
        ),
        Action::Return { value } => format!("return {value:?}"),
        Action::Switch {
            discriminant,
            arms,
            default_body,
        } => format!(
            "switch {discriminant:?} {} arms, default {} actions",
            arms.len(),
            default_body.len()
        ),
        Action::Break => "break".into(),
        Action::Continue => "continue".into(),
        Action::Try {
            try_body,
            catch_body,
        } => format!(
            "try {} / catch {} actions",
            try_body.len(),
            catch_body.len()
        ),
        Action::Expr { name, value } => format!("expr {name} = {value:?}"),
        Action::Async { body } => format!("async block {} actions", body.len()),
        Action::Module { name, actions } => format!("module {name} ({} actions)", actions.len()),
        Action::Const { name, value } => format!("const {name} = {value:?}"),
        Action::ListStore { name, items } => format!("list {name} [{} items]", items.len()),
        Action::Throw { message } => format!("throw {message:?}"),
        Action::Await { binding } => format!("await {:?}", binding),
        Action::Call { name, args, into } => {
            format!("call {name}({} args) -> {:?}", args.len(), into)
        }
    }
}
