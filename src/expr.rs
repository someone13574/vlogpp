use std::fmt::{self, Display};

use crate::r#macro::MacroID;
use crate::scope::Scope;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct VarID(pub usize);

pub struct Var {
    pub id: VarID,
    pub name: String,
}

impl Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(VarID),
    Macro(MacroID),
    Text(String),
    List(Vec<Expr>),
    Concat(Vec<Expr>),
    Call { r#macro: Box<Expr>, args: Vec<Expr> },
}

impl Expr {
    pub fn vars(&self) -> Vec<VarID> {
        match self {
            Expr::Var(var_id) => vec![*var_id],
            Expr::Macro(_) | Expr::Text(_) => Vec::new(),
            Expr::List(exprs) | Expr::Concat(exprs) => {
                exprs
                    .iter()
                    .flat_map(|expr| expr.vars().into_iter())
                    .collect()
            }
            Expr::Call { r#macro, args } => {
                r#macro
                    .vars()
                    .into_iter()
                    .chain(args.iter().flat_map(|expr| expr.vars().into_iter()))
                    .collect()
            }
        }
    }

    pub fn emit(&self, scope: Scope) -> String {
        match self {
            Expr::Var(var_id) => scope.get_var(*var_id).name.clone(),
            Expr::Macro(macro_id) => scope.get_macro(*macro_id).name.clone(),
            Expr::Text(text) => text.clone(),
            Expr::List(exprs) => {
                exprs
                    .iter()
                    .map(|expr| expr.emit(scope))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            Expr::Concat(exprs) => {
                exprs
                    .iter()
                    .map(|expr| expr.emit(scope))
                    .collect::<Vec<_>>()
                    .join("##")
            }
            Expr::Call { r#macro, args } => {
                format!(
                    "{}({})",
                    r#macro.emit(scope),
                    args.iter()
                        .map(|expr| expr.emit(scope))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}
