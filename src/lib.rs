use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::{collections::HashMap, fmt, sync::Arc};

pub trait FunctionBuilder {
    fn build(&self, args: Vec<Expr>) -> Arc<dyn Function>;
}

pub trait Function {
    fn dependencies(&self) -> Vec<String>;
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum StaticExpr {
    Number(f64),
    String(String),
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ExprData {
    Static(StaticExpr),
    Dynamic(FunctionExprData),
}

#[derive(Default)]
pub struct Registry {
    fns: HashMap<String, Arc<dyn FunctionBuilder>>,
}

impl Registry {
    pub fn insert(&mut self, id: impl Into<String>, builder: impl FunctionBuilder + 'static) {
        self.fns.insert(id.into(), Arc::new(builder));
    }
}

impl ExprData {
    pub fn build(self, registry: &Registry) -> Expr {
        match self {
            ExprData::Static(s) => Expr::Static(s),
            ExprData::Dynamic(fn_expr) => {
                let builder = registry.fns.get(&fn_expr.ident).unwrap();

                let args = fn_expr
                    .args
                    .into_iter()
                    .map(|arg| arg.build(registry))
                    .collect();
                Expr::Dynamic(FunctionExpr {
                    f: builder.build(args),
                })
            }
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Static(StaticExpr),
    Dynamic(FunctionExpr),
}

impl Expr {
    pub fn deps(&self) -> Vec<String> {
        match self {
            Expr::Static(_) => Vec::new(),
            Expr::Dynamic(fn_expr) => fn_expr.f.dependencies(),
        }
    }
}

pub struct FunctionExpr {
    pub f: Arc<dyn Function>,
}

impl fmt::Debug for FunctionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionExpr").finish()
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

#[cfg(test)]
mod tests {
    use super::*;

    struct A;

    impl FunctionBuilder for A {
        fn build(&self, args: Vec<Expr>) -> Arc<dyn Function> {
            Arc::new(B { args })
        }
    }

    struct B {
        args: Vec<Expr>,
    }

    impl Function for B {
        fn dependencies(&self) -> Vec<String> {
            self.args.iter().map(|arg| arg.deps()).flatten().collect()
        }
    }

    #[test]
    fn it_works() {
        let mut registry = Registry::default();
        registry.insert("add", A);

        let data: ExprData = serde_json::from_str(r#" [ "add", 1, 2 ] "#).unwrap();
        let expr = data.build(&registry);
        dbg!(expr.deps());
    }
}
