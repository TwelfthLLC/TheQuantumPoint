use ir::{emit_value_expr, sanitize_ident, Action, ValueExpr};
use std::fmt::Write;

pub(crate) fn emit_action(out: &mut String, action: &Action, indent: usize) {
    let pad = "    ".repeat(indent);
    match action {
        Action::Print { message } => {
            writeln!(out, "{pad}println!(\"{}\");", escape_rust_str(message)).unwrap();
        }
        Action::DataStore { name, value } => {
            let ident = sanitize_ident(name);
            let rhs = infer_rhs(value);
            writeln!(out, "{pad}let {ident} = {rhs};").unwrap();
        }
        Action::Const { name, value } => {
            let ident = sanitize_ident(name);
            let rhs = infer_rhs(value);
            writeln!(out, "{pad}const {ident} = {rhs};").unwrap();
        }
        Action::ListStore { name, items } => {
            let ident = sanitize_ident(name);
            let elems: Vec<String> = items.iter().map(infer_rhs).collect();
            writeln!(out, "{pad}let {ident} = vec![{}];", elems.join(", ")).unwrap();
        }
        Action::Throw { message } => {
            writeln!(out, "{pad}return Err({message:?}.to_string());").unwrap();
        }
        Action::Await { binding } => {
            writeln!(out, "{pad}tokio::task::yield_now().await;").unwrap();
            if let Some(b) = binding {
                let ident = sanitize_ident(b);
                writeln!(out, "{pad}let {ident} = ();").unwrap();
            }
        }
        Action::Call { name, args, into } => {
            let fname = sanitize_ident(name);
            let arglist: Vec<String> = args.iter().map(emit_value_expr).collect();
            let call = format!("{fname}({})", arglist.join(", "));
            if let Some(var) = into {
                let ident = sanitize_ident(var);
                writeln!(out, "{pad}let {ident} = {call};").unwrap();
            } else {
                writeln!(out, "{pad}{call};").unwrap();
            }
        }
        Action::DbRead { table, into_var } => {
            let ident = sanitize_ident(into_var);
            writeln!(
                out,
                "{pad}let {ident} = qp_mock_rows({:?}).into_iter().next().unwrap_or((\"id\", \"0\"));",
                table
            )
            .unwrap();
            writeln!(
                out,
                "{pad}println!(\"db.read {{}} -> {{:?}}\", {:?}, {ident});",
                table
            )
            .unwrap();
        }
        Action::Module { name, actions } => {
            writeln!(out, "{pad}{{ // module {name}").unwrap();
            for a in actions {
                emit_action(out, a, indent + 1);
            }
            writeln!(out, "{pad}}}").unwrap();
        }
        Action::Branch {
            condition,
            then_body,
            else_body,
        } => {
            let cond = emit_value_expr(condition);
            writeln!(out, "{pad}if {cond} {{").unwrap();
            for a in then_body {
                emit_action(out, a, indent + 1);
            }
            if else_body.is_empty() {
                writeln!(out, "{pad}}}").unwrap();
            } else {
                writeln!(out, "{pad}}} else {{").unwrap();
                for a in else_body {
                    emit_action(out, a, indent + 1);
                }
                writeln!(out, "{pad}}}").unwrap();
            }
        }
        Action::While { condition, body } => {
            let cond = emit_value_expr(condition);
            writeln!(out, "{pad}while {cond} {{").unwrap();
            for a in body {
                emit_action(out, a, indent + 1);
            }
            writeln!(out, "{pad}}}").unwrap();
        }
        Action::For {
            var,
            from,
            to,
            body,
        } => {
            let ident = sanitize_ident(var);
            writeln!(out, "{pad}for {ident} in {from}..={to} {{").unwrap();
            for a in body {
                emit_action(out, a, indent + 1);
            }
            writeln!(out, "{pad}}}").unwrap();
        }
        Action::ForEach {
            item_var,
            collection,
            body,
        } => {
            let item = sanitize_ident(item_var);
            writeln!(out, "{pad}for {item} in qp_mock_rows({:?}) {{", collection).unwrap();
            for a in body {
                emit_action(out, a, indent + 1);
            }
            writeln!(out, "{pad}}}").unwrap();
        }
        Action::Return { value } => {
            if let Some(v) = value {
                writeln!(out, "{pad}return {};", infer_rhs(v)).unwrap();
            } else {
                writeln!(out, "{pad}return;").unwrap();
            }
        }
        Action::Switch {
            discriminant,
            arms,
            default_body,
        } => {
            let disc = emit_value_expr(discriminant);
            writeln!(out, "{pad}match &{disc} {{").unwrap();
            for arm in arms {
                let pat = switch_arm_pattern(&arm.label);
                writeln!(out, "{pad}    {pat} => {{").unwrap();
                for a in &arm.body {
                    emit_action(out, a, indent + 2);
                }
                writeln!(out, "{pad}    }}").unwrap();
            }
            writeln!(out, "{pad}    _ => {{").unwrap();
            for a in default_body {
                emit_action(out, a, indent + 2);
            }
            writeln!(out, "{pad}    }}").unwrap();
            writeln!(out, "{pad}}}").unwrap();
        }
        Action::Break => {
            writeln!(out, "{pad}break;").unwrap();
        }
        Action::Continue => {
            writeln!(out, "{pad}continue;").unwrap();
        }
        Action::Try {
            try_body,
            catch_body,
        } => {
            writeln!(out, "{pad}match (|| -> Result<(), String> {{").unwrap();
            for a in try_body {
                emit_action(out, a, indent + 1);
            }
            writeln!(out, "{pad}    Ok(())").unwrap();
            writeln!(out, "{pad}}})() {{").unwrap();
            writeln!(out, "{pad}    Ok(()) => {{}},").unwrap();
            writeln!(out, "{pad}    Err(_e) => {{").unwrap();
            for a in catch_body {
                emit_action(out, a, indent + 2);
            }
            writeln!(out, "{pad}    }}").unwrap();
            writeln!(out, "{pad}}}").unwrap();
        }
        Action::Expr { name, value } => {
            let ident = sanitize_ident(name);
            writeln!(out, "{pad}let {ident} = {};", emit_value_expr(value)).unwrap();
        }
        Action::Async { body } => {
            writeln!(out, "{pad}{{").unwrap();
            for a in body {
                emit_action(out, a, indent + 1);
            }
            writeln!(out, "{pad}}}").unwrap();
        }
    }
}

fn switch_arm_pattern(label: &str) -> String {
    if let Ok(n) = label.parse::<i64>() {
        format!("{n}")
    } else if label == "true" || label == "false" {
        label.to_string()
    } else {
        format!("{label:?}")
    }
}

fn infer_rhs(value: &ValueExpr) -> String {
    match value {
        ValueExpr::Bool(b) => b.to_string(),
        ValueExpr::I64(n) => n.to_string(),
        ValueExpr::F64(n) => format!("{n}"),
        ValueExpr::Str(s) => format!("{s:?}"),
        ValueExpr::Ident(name) => sanitize_ident(name),
        other => emit_value_expr(other),
    }
}

fn escape_rust_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
