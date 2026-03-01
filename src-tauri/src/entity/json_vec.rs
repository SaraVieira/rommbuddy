use sea_orm::entity::prelude::*;
use sea_orm::{sea_query, TryGetError, TryGetable, Value};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Newtype wrapping `Vec<String>` for columns stored as JSON TEXT in SQLite.
///
/// Serializes to/from JSON arrays (e.g. `["a","b"]`). An empty or null column
/// deserializes to an empty vec.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JsonVec(pub Vec<String>);

impl JsonVec {
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}

impl From<Vec<String>> for JsonVec {
    fn from(v: Vec<String>) -> Self {
        Self(v)
    }
}

impl fmt::Display for JsonVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.0).unwrap_or_default())
    }
}

impl From<JsonVec> for Value {
    fn from(v: JsonVec) -> Self {
        Value::String(Some(Box::new(
            serde_json::to_string(&v.0).unwrap_or_else(|_| "[]".to_string()),
        )))
    }
}

impl TryGetable for JsonVec {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let val: Option<String> = res.try_get_by(idx)?;
        Ok(match val {
            Some(s) if !s.is_empty() => {
                JsonVec(serde_json::from_str(&s).unwrap_or_default())
            }
            _ => JsonVec::default(),
        })
    }
}

impl sea_query::ValueType for JsonVec {
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        match v {
            Value::String(Some(s)) => {
                Ok(JsonVec(serde_json::from_str(&s).unwrap_or_default()))
            }
            Value::String(None) => Ok(JsonVec::default()),
            _ => Err(sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "JsonVec".to_string()
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::String
    }

    fn column_type() -> sea_query::ColumnType {
        sea_query::ColumnType::Text
    }
}

impl sea_orm::sea_query::Nullable for JsonVec {
    fn null() -> Value {
        Value::String(None)
    }
}
