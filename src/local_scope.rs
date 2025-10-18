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

    pub input_map: HashMap<String, VarID>,
    pub output_names: Option<Vec<String>>,
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
    pub fn new_var(&mut self, name: &str, map_input: bool) -> VarID {
        let mut alias;
        let mut suffix = None;

        while {
            alias = if let Some(suffix) = suffix {
                format!("{}{suffix}", preprocess_var_name(name))
            } else {
                preprocess_var_name(name)
            };

            self.vars.values().any(|existing| existing.name == alias)
        } {
            suffix = Some(suffix.map_or(0, |x| x + 1));
        }

        let id = self.next_var_id;
        self.next_var_id.0 += 1;
        self.vars.insert(id, Var { id, name: alias });
        if map_input {
            assert_eq!(self.input_map.insert(name.to_string(), id), None);
        }

        id
    }

    pub fn new_expr(
        &mut self,
        content: ExprContent,
        wrapper: Option<(MacroID, Option<ExprID>)>,
    ) -> ExprID {
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

    pub fn new_var_expr(
        &mut self,
        name: &str,
        map_input: bool,
        wrapper: Option<(MacroID, Option<ExprID>)>,
    ) -> (VarID, ExprID) {
        let var = self.new_var(name, map_input);
        let expr = self.new_expr(ExprContent::Var(var), wrapper);

        (var, expr)
    }
}

fn preprocess_var_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .map(|c| c.to_ascii_lowercase())
        .collect::<String>()
}
