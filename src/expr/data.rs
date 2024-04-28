use super::{Expr, StaticExpr};
use crate::{Registry, Scope};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ExprData {
    Static(StaticExpr),
    Dynamic(FunctionExprData),
}

impl ExprData {
    pub fn build(self, registry: &Registry) -> Scope {
        let expr = self.build_expr(registry);
        let dependencies = expr.deps().into_iter().map(|id| (id, None)).collect();
        Scope { expr, dependencies }
    }

    pub fn build_expr(self, registry: &Registry) -> Expr {
        match self {
            ExprData::Static(s) => Expr::Static(s),
            ExprData::Dynamic(fn_expr) => {
                let builder = registry.fns.get(&fn_expr.ident).unwrap();

                let args = fn_expr
                    .args
                    .into_iter()
                    .map(|arg| arg.build_expr(registry))
                    .collect();
                Expr::Dynamic(builder.dyn_build(args))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionExprData {
    pub ident: String,
    pub args: Vec<ExprData>,
}

impl<'de> Deserialize<'de> for FunctionExprData {
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

            Ok(FunctionExprData { ident, args })
        } else {
            Err(serde::de::Error::custom("Expected array."))
        }
    }
}