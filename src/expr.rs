use std::fmt::{self, Display};

use crate::{
    r#macro::MacroID,
    scope::{GlobalScope, LocalScope},
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct VarID(pub usize);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ExprID(pub usize);

pub struct Var {
    pub id: VarID,
    pub name: String,
}

impl Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub enum ExprContent {
    Var(VarID),
    Text(String),
    List(Vec<ExprID>),
    Concat(Vec<VarID>),
}

pub struct Expr {
    pub id: ExprID,
    pub wrapper: Option<MacroID>,
    pub content: ExprContent,
}

impl Expr {
    pub fn emit(&self, global_scope: &GlobalScope, local_scope: &LocalScope) -> String {
        let content = match &self.content {
            ExprContent::Var(var_id) => {
                format!("{}", local_scope.get_var(var_id))
            }
            ExprContent::Text(text) => {
                format!("{text}")
            }
            ExprContent::List(expr_ids) => {
                format!(
                    "{}",
                    expr_ids
                        .iter()
                        .map(|expr_id| {
                            format!(
                                "{}",
                                local_scope
                                    .get_expr(expr_id)
                                    .emit(global_scope, local_scope)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ExprContent::Concat(var_ids) => {
                format!(
                    "{}",
                    var_ids
                        .iter()
                        .map(|var_id| { format!("{}", local_scope.get_var(var_id)) })
                        .collect::<Vec<_>>()
                        .join("##")
                )
            }
        };

        if let Some(wrapper) = &self.wrapper {
            format!(
                "{}({content})",
                global_scope.macros.get(wrapper).unwrap().name
            )
        } else {
            content
        }
    }
}
