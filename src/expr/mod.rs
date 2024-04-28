use crate::Scope;
use serde::Deserialize;
use std::{fmt, sync::Arc};

mod data;
pub use self::data::{ExprData, FunctionExprData};

pub mod function;
use self::function::Function;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum StaticExpr {
    Number(f64),
    String(String),
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
