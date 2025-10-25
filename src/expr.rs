use colored::Colorize;

use crate::r#macro::MacroID;
use crate::scope::Scope;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct VarID(pub usize);

pub struct Var {
    pub id: VarID,
    pub name: String,
    pub variadic: bool,
    pub bundle_id: Option<VarID>,
}

impl Var {
    pub fn input_text(&self) -> &str {
        if self.variadic { "..." } else { &self.name }
    }

    pub fn expr_text(&self) -> &str {
        if self.variadic {
            "__VA_ARGS__"
        } else {
            &self.name
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Var(VarID),
    Macro(MacroID),
    Text(String),
    List(Vec<Expr>, &'static str),
    Call { r#macro: Box<Expr>, args: Vec<Expr> },
}

impl Expr {
    pub fn vars(&self) -> Vec<VarID> {
        match self {
            Expr::Var(var_id) => vec![*var_id],
            Expr::Macro(_) | Expr::Text(_) => Vec::new(),
            Expr::List(exprs, _sep) => {
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

    pub fn var_spans(&self) -> Vec<Vec<VarID>> {
        match self {
            Expr::Var(var_id) => vec![vec![*var_id]],
            Expr::Macro(_) | Expr::Text(_) | Expr::List(_, "##") => Vec::new(),
            Expr::List(exprs, _)
            | Expr::Call {
                r#macro: _,
                args: exprs,
            } => {
                let mut spans = Vec::new();
                for start in 0..exprs.len() {
                    let mut curr_span = Vec::new();
                    for end_inc in start..exprs.len() {
                        if let Expr::Var(var_id) = exprs.get(end_inc).unwrap() {
                            curr_span.push(*var_id);
                            spans.push(curr_span.clone());
                        } else {
                            spans.extend(exprs.get(end_inc).unwrap().var_spans());
                            break;
                        }
                    }
                }

                spans
            }
        }
    }

    pub fn emit(&self, scope: Scope) -> String {
        match self {
            Expr::Var(var_id) => scope.get_var(*var_id).expr_text().to_string(),
            Expr::Macro(macro_id) => scope.get_macro(*macro_id).name.magenta().to_string(),
            Expr::Text(text) => text.green().to_string(),
            Expr::List(exprs, sep) => {
                exprs
                    .iter()
                    .map(|expr| expr.emit(scope))
                    .collect::<Vec<_>>()
                    .join(sep)
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
