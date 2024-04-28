use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::{any::Any, collections::HashMap, fmt, sync::Arc};

pub trait FunctionBuilder {
    fn build(&self, args: Vec<Expr>) -> Arc<dyn Function>;
}

pub trait Function {
    fn dependencies(&self) -> Vec<String>;

    fn run(&self, scope: &Scope) -> f64;
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
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
    pub fn run(&self, scope: &Scope) -> StaticExpr {
        match self {
            Expr::Static(s) => s.clone(),
            Expr::Dynamic(fn_expr) => StaticExpr::Number(fn_expr.f.run(scope)),
        }
    }

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

pub struct Scope {
    expr: Expr,
    dependencies: HashMap<String, Option<f64>>,
}

impl Scope {
    pub fn set_dependency(&mut self, id: &str, value: f64) {
        *self.dependencies.get_mut(id).unwrap() = Some(value);
    }

    pub fn run(&self) -> Option<StaticExpr> {
        if !self.dependencies.values().all(Option::is_some) {
            return None;
        }

        Some(self.expr.run(self))
    }
}

struct AddFunctionBuilder;

impl FunctionBuilder for AddFunctionBuilder {
    fn build(&self, args: Vec<Expr>) -> Arc<dyn Function> {
        Arc::new(AddFunction { args })
    }
}

struct AddFunction {
    args: Vec<Expr>,
}

impl Function for AddFunction {
    fn dependencies(&self) -> Vec<String> {
        self.args.iter().map(|arg| arg.deps()).flatten().collect()
    }

    fn run(&self, scope: &Scope) -> f64 {
        let mut sum = 0.;

        for arg in &self.args {
            if let StaticExpr::Number(n) = arg.run(scope) {
                sum += n
            } else {
                todo!()
            }
        }

        sum
    }
}

struct QueryFunctionBuilder;

impl FunctionBuilder for QueryFunctionBuilder {
    fn build(&self, args: Vec<Expr>) -> Arc<dyn Function> {
        if let Some(Expr::Static(StaticExpr::String(s))) = args.first() {
            Arc::new(QueryFunction {
                dependency: s.clone(),
            })
        } else {
            todo!()
        }
    }
}

struct QueryFunction {
    dependency: String,
}

impl Function for QueryFunction {
    fn dependencies(&self) -> Vec<String> {
        vec![self.dependency.clone()]
    }

    fn run(&self, scope: &Scope) -> f64 {
        scope.dependencies.get(&self.dependency).unwrap().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut registry = Registry::default();
        registry.insert("+", AddFunctionBuilder);
        registry.insert("@", QueryFunctionBuilder);

        let data: ExprData = serde_json::from_str(r#" [ "+", [ "@", "test" ], 2 ] "#).unwrap();
        let mut scope = data.build(&registry);
        dbg!(scope.run());

        scope.set_dependency("test", 3.);
        dbg!(scope.run());
    }
}
