use ir::{Action, FunctionDef};
use std::collections::HashMap;

use super::flow::Flow;
use super::mock::mock_collection;
use super::value::{eval_value, runtime_to_switch_key, RuntimeValue};
use super::RuntimeError;

pub(crate) fn run_actions(
    actions: &[Action],
    env: &mut HashMap<String, RuntimeValue>,
    lines: &mut Vec<String>,
    in_loop: bool,
    functions: &HashMap<&str, &FunctionDef>,
) -> Result<Flow, RuntimeError> {
    for action in actions {
        match run_action(action, env, lines, in_loop, functions)? {
            Flow::Next => {}
            other => return Ok(other),
        }
    }
    Ok(Flow::Next)
}

fn run_action(
    action: &Action,
    env: &mut HashMap<String, RuntimeValue>,
    lines: &mut Vec<String>,
    in_loop: bool,
    functions: &HashMap<&str, &FunctionDef>,
) -> Result<Flow, RuntimeError> {
    match action {
        Action::Print { message } => {
            lines.push(message.clone());
            Ok(Flow::Next)
        }
        Action::DataStore { name, value } => {
            let v = eval_value(value, env)?;
            env.insert(name.clone(), v);
            Ok(Flow::Next)
        }
        Action::Const { name, value } => {
            let v = eval_value(value, env)?;
            env.insert(name.clone(), v);
            lines.push(format!("const {name} = {:?}", env.get(name)));
            Ok(Flow::Next)
        }
        Action::ListStore { name, items } => {
            let vals: Result<Vec<_>, _> = items.iter().map(|i| eval_value(i, env)).collect();
            let list = RuntimeValue::List(vals?);
            env.insert(name.clone(), list);
            Ok(Flow::Next)
        }
        Action::Throw { message } => Err(RuntimeError::Message(message.clone())),
        Action::Await { binding } => {
            lines.push("await".into());
            if let Some(b) = binding {
                env.insert(b.clone(), RuntimeValue::Str("()".into()));
            }
            Ok(Flow::Next)
        }
        Action::Call { name, args, into } => {
            let f = functions
                .get(name.as_str())
                .ok_or_else(|| RuntimeError::UnknownFn(name.clone()))?;
            let mut local = env.clone();
            for (param, arg) in f.params.iter().zip(args.iter()) {
                local.insert(param.clone(), eval_value(arg, &local)?);
            }
            let mut sub_lines = Vec::new();
            let flow = run_actions(&f.body, &mut local, &mut sub_lines, false, functions)?;
            for line in sub_lines {
                lines.push(format!("  {line}"));
            }
            if let Flow::Return(Some(v)) = flow {
                if let Some(var) = into {
                    env.insert(var.clone(), v);
                }
            } else if let Some(var) = into {
                env.insert(var.clone(), RuntimeValue::I64(0));
            }
            lines.push(format!("call {name}"));
            Ok(Flow::Next)
        }
        Action::Branch {
            condition,
            then_body,
            else_body,
        } => {
            if eval_value(condition, env)?.as_bool() {
                run_actions(then_body, env, lines, in_loop, functions)
            } else {
                run_actions(else_body, env, lines, in_loop, functions)
            }
        }
        Action::While { condition, body } => {
            while eval_value(condition, env)?.as_bool() {
                match run_actions(body, env, lines, true, functions)? {
                    Flow::Next => {}
                    Flow::Break => break,
                    Flow::Continue => continue,
                    other => return Ok(other),
                }
            }
            Ok(Flow::Next)
        }
        Action::ForEach {
            item_var,
            collection,
            body,
        } => {
            for row in mock_collection(collection) {
                env.insert(item_var.clone(), row);
                match run_actions(body, env, lines, true, functions)? {
                    Flow::Next => {}
                    Flow::Break => break,
                    Flow::Continue => continue,
                    other => return Ok(other),
                }
            }
            Ok(Flow::Next)
        }
        Action::For {
            var,
            from,
            to,
            body,
        } => {
            let from = *from;
            let to = *to;
            let step = if from <= to { 1 } else { -1 };
            let mut i = from;
            loop {
                if step > 0 && i > to {
                    break;
                }
                if step < 0 && i < to {
                    break;
                }
                env.insert(var.clone(), RuntimeValue::I64(i));
                match run_actions(body, env, lines, true, functions)? {
                    Flow::Next => {}
                    Flow::Break => break,
                    Flow::Continue => {
                        i += step;
                        continue;
                    }
                    other => return Ok(other),
                }
                i += step;
            }
            Ok(Flow::Next)
        }
        Action::Return { value } => {
            let rv = value.as_ref().map(|v| eval_value(v, env)).transpose()?;
            if let Some(v) = &rv {
                lines.push(format!("return {v:?}"));
            } else {
                lines.push("return".into());
            }
            Ok(Flow::Return(rv))
        }
        Action::Switch {
            discriminant,
            arms,
            default_body,
        } => {
            let key = runtime_to_switch_key(&eval_value(discriminant, env)?);
            for arm in arms {
                if arm.label == key {
                    return run_actions(&arm.body, env, lines, in_loop, functions);
                }
            }
            run_actions(default_body, env, lines, in_loop, functions)
        }
        Action::Break => {
            if in_loop {
                Ok(Flow::Break)
            } else {
                Err(RuntimeError::BreakOutsideLoop)
            }
        }
        Action::Continue => {
            if in_loop {
                Ok(Flow::Continue)
            } else {
                Err(RuntimeError::ContinueOutsideLoop)
            }
        }
        Action::Try {
            try_body,
            catch_body,
        } => {
            lines.push("try {".into());
            let try_flow = run_actions(try_body, env, lines, in_loop, functions);
            if matches!(
                try_flow,
                Err(RuntimeError::Message(_)) | Err(RuntimeError::UnknownVar(_))
            ) {
                lines.push("} catch {".into());
                run_actions(catch_body, env, lines, in_loop, functions)
            } else {
                lines.push("} // try ok".into());
                try_flow
            }
        }
        Action::Expr { name, value } => {
            let v = eval_value(value, env)?;
            env.insert(name.clone(), v);
            Ok(Flow::Next)
        }
        Action::Async { body } => {
            lines.push("async { ... }".into());
            run_actions(body, env, lines, in_loop, functions)
        }
        Action::DbRead { table, into_var } => {
            let row = mock_collection(table)
                .into_iter()
                .next()
                .unwrap_or(RuntimeValue::Str("empty".into()));
            env.insert(into_var.clone(), row.clone());
            lines.push(format!("db.read {table} -> {row:?}"));
            Ok(Flow::Next)
        }
        Action::Module { name, actions } => {
            lines.push(format!("— module {name} —"));
            run_actions(actions, env, lines, in_loop, functions)
        }
    }
}
