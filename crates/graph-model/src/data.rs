use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Typed node field — binar `.qp` ichida postcard bilan saqlanadi (JSON fayl emas).
/// Postcard: oddiy enum variantlari (ichki `tag`/`content` ishlatilmaydi).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
    /// Assign / expression: `Typed { ty: "i64", value: I64(1) }`
    Typed {
        ty: String,
        value: Box<DataValue>,
    },
}

impl Default for DataValue {
    fn default() -> Self {
        DataValue::Str(String::new())
    }
}

impl DataValue {
    pub fn str(s: impl Into<String>) -> Self {
        DataValue::Str(s.into())
    }

    pub fn i64(n: i64) -> Self {
        DataValue::I64(n)
    }

    pub fn typed_i64(n: i64) -> Self {
        DataValue::Typed {
            ty: "i64".to_string(),
            value: Box::new(DataValue::I64(n)),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            DataValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            DataValue::I64(n) => Some(*n),
            DataValue::Typed { value, .. } => value.as_i64(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            DataValue::Bool(b) => Some(*b),
            DataValue::Typed { value, .. } => value.as_bool(),
            _ => None,
        }
    }
}

pub type NodeData = HashMap<String, DataValue>;

pub fn data_get_str(data: &NodeData, key: &str) -> Option<String> {
    data.get(key).and_then(|v| v.as_str().map(str::to_string))
}

pub fn data_get_i64(data: &NodeData, key: &str) -> Option<i64> {
    data.get(key).and_then(|v| v.as_i64())
}

pub fn data_set_str(data: &mut NodeData, key: &str, val: &str) {
    data.insert(key.to_string(), DataValue::str(val));
}
