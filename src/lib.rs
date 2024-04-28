use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum StaticExpr {
    Number(f64),
    String(String),
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Expr {
    Static(StaticExpr),
    Dynamic(FunctionExpr),
}

#[derive(Debug, PartialEq)]
pub struct FunctionExpr {
    pub ident: String,
    pub args: Vec<Expr>,
}

impl<'de> Deserialize<'de> for FunctionExpr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        if let Value::Array(items) = value {
            let ident = items[0].as_str().ok_or_else(|| {
                serde::de::Error::custom("Expected string for function identifier.")
            })?;

            let ident = ident.to_owned();
            let args = items[1..]
                .iter()
                .map(|v| serde_json::from_value(v.clone()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| serde::de::Error::custom("Invalid JSON."))?;

            Ok(FunctionExpr { ident, args })
        } else {
            Err(serde::de::Error::custom("Expected array."))
        }
    }
}
