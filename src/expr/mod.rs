use crate::ScopeData;

use serde::Deserialize;
use std::sync::Arc;

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

pub enum Expr {
    Static(StaticExpr),
    Dynamic(Arc<dyn Function>),
}

impl Expr {
    pub fn run(&self, scope: &ScopeData) -> StaticExpr {
        match self {
            Expr::Static(static_expr) => static_expr.clone(),
            Expr::Dynamic(fn_expr) => StaticExpr::Number(fn_expr.run(scope)),
        }
    }

    pub fn deps(&self) -> Vec<String> {
        match self {
            Expr::Static(_) => Vec::new(),
            Expr::Dynamic(fn_expr) => fn_expr.dependencies(),
        }
    }
}
