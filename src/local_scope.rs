use std::collections::HashMap;

use crate::expr::{Expr, ExprContent, ExprID, Var, VarID};
use crate::r#macro::MacroID;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct LocalScopeID(pub usize);

pub struct LocalScope {
    pub next_var_id: VarID,
    pub next_expr_id: ExprID,

    pub vars: HashMap<VarID, Var>,
    pub exprs: HashMap<ExprID, Expr>,
}

impl LocalScope {
    pub fn get_var(&self, var_id: VarID) -> &Var {
        self.vars.get(&var_id).unwrap()
    }

    pub fn get_expr(&self, expr_id: ExprID) -> &Expr {
        self.exprs.get(&expr_id).unwrap()
    }
}

impl LocalScope {
    pub fn new_var(&mut self, name: &str) -> VarID {
        let mut alias;
        let mut suffix = None;

        while {
            alias = if let Some(suffix) = suffix {
                format!("{}__{suffix}", name.to_lowercase())
            } else {
                name.to_uppercase()
            };

            self.vars.values().any(|existing| existing.name == alias)
        } {
            suffix = Some(suffix.map_or(0, |x| x + 1));
        }

        let id = self.next_var_id;
        self.next_var_id.0 += 1;

        self.vars.insert(id, Var { id, name: alias });
        id
    }

    pub fn new_expr(&mut self, content: ExprContent, wrapper: Option<MacroID>) -> ExprID {
        let id = self.next_expr_id;
        self.next_expr_id.0 += 1;

        self.exprs.insert(
            id,
            Expr {
                id,
                wrapper,
                content,
            },
        );
        id
    }
}
