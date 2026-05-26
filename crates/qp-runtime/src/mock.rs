use super::value::RuntimeValue;

pub(crate) fn mock_collection(table: &str) -> Vec<RuntimeValue> {
    match table {
        "users" => vec![
            RuntimeValue::Str("user:1:Ada".into()),
            RuntimeValue::Str("user:2:Bob".into()),
        ],
        "orders" => vec![
            RuntimeValue::Str("order:100".into()),
            RuntimeValue::Str("order:101".into()),
        ],
        other => vec![RuntimeValue::Str(format!("row-from-{other}"))],
    }
}
