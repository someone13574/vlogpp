use colored::Colorize;

use crate::expr::{Expr, VarID};
use crate::scope::global::GlobalScope;
use crate::scope::local::LocalScopeID;
use crate::scope::{MutScope, Scope};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct MacroID(pub usize);

pub struct Macro {
    pub scope_id: LocalScopeID,

    pub name: String,
    pub expr: Expr,

    pub inputs: Vec<VarID>,
    pub calling_split: Option<MacroID>,
    pub output_to_input: Option<usize>,

    pub doc_name: Option<String>,
}

impl Macro {
    pub fn input_position(&self, name: &str, global_scope: &GlobalScope) -> Option<usize> {
        let scope = global_scope.get_scope(self.scope_id);
        let var_id = scope.local().input_map.get(name).unwrap();
        self.inputs.iter().position(|input| input == var_id)
    }

    pub fn check_inputs(
        &self,
        indices: Vec<usize>,
        global_scope: &GlobalScope,
    ) -> Result<(), String> {
        let scope = global_scope.get_scope(self.scope_id);

        for (idx, input) in self.inputs.iter().enumerate() {
            if indices
                .get(idx)
                .is_none_or(|provided_idx| idx < *provided_idx)
            {
                return Err(format!(
                    "Missing variable `{}` for macro `{}`",
                    scope.get_var(*input).name,
                    &self.name
                ));
            } else if idx > *indices.get(idx).unwrap() {
                return Err(format!(
                    "Duplicate variable `{}` for macro `{}`",
                    scope.get_var(*input).name,
                    &self.name
                ));
            }
        }

        assert_eq!(self.inputs.len(), indices.len());

        Ok(())
    }

    pub fn sort_passthrough_vars(id: MacroID, scope: &mut MutScope) {
        let mut input_perm = scope
            .get_macro(id)
            .inputs
            .iter()
            .enumerate()
            .map(|(idx, input)| {
                (
                    idx,
                    scope
                        .get_macro(id)
                        .var_passthrough_score(*input, &scope.scope()),
                )
            })
            .collect::<Vec<_>>();
        input_perm.sort_by_key(|(_, score)| *score);

        let inputs = &mut scope.get_mut_macro(id).inputs;
        *inputs = input_perm
            .iter()
            .map(|(idx, _)| *inputs.get(*idx).unwrap())
            .collect();

        if let Some(caller) = scope
            .get_macro(id)
            .calling_split
            .iter()
            .find(|caller| scope.get_macro(**caller).scope_id == scope.id)
            .copied()
        {
            let Expr::Call { r#macro: _, args } = &mut scope.get_mut_macro(caller).expr else {
                unreachable!();
            };

            *args = input_perm
                .iter()
                .map(|(idx, _)| args.get(*idx).unwrap().clone())
                .collect();
        }
    }

    fn var_passthrough_score(&self, id: VarID, scope: &Scope) -> (usize, usize) {
        let (caller_score, caller_max_score) = if let Some(caller) = self
            .calling_split
            .iter()
            .find(|caller| scope.get_macro(**caller).scope_id == scope.id)
        {
            scope.get_macro(*caller).var_passthrough_score(id, scope)
        } else {
            (0, 1)
        };

        let max_expr_score = self.expr.vars().len();
        let expr_score = self
            .expr
            .vars()
            .iter()
            .position(|var| *var == id)
            .unwrap_or_default();
        let indiv_score = if self.is_passthrough_var(id) {
            max_expr_score + expr_score
        } else {
            0
        };
        let max_indiv_score = max_expr_score * 2;

        (
            caller_score + indiv_score * caller_max_score,
            caller_max_score + max_indiv_score * caller_max_score,
        )
    }

    fn is_passthrough_var(&self, id: VarID) -> bool {
        let mut count = 0;
        let exprs = match &self.expr {
            Expr::List(exprs, _) => exprs,
            Expr::Call { r#macro: _, args } => args,
            _ => unreachable!(),
        };

        for expr in exprs {
            let count_in_expr = expr.vars().iter().filter(|var| **var == id).count();
            if count_in_expr != 0 && *expr != Expr::Var(id) {
                return false;
            }

            count += count_in_expr;
        }

        count == 1
    }

    pub fn emit(&self, global_scope: &GlobalScope) -> String {
        let scope = global_scope.get_scope(self.scope_id);
        let docs = if let Some(doc_name) = &self.doc_name {
            if cfg!(feature = "obfuscate") {
                String::new()
            } else {
                format!(
                    "// Module: `{doc_name}`, Inputs: {}, Outputs: {}\n",
                    self.inputs
                        .iter()
                        .map(|var_id| {
                            scope
                                .local()
                                .input_map
                                .iter()
                                .find(|(_, id)| *id == var_id)
                                .unwrap()
                                .0
                                .as_str()
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                    scope.local().output_names.as_ref().unwrap().join(", ")
                )
            }
        } else {
            String::new()
        };

        format!(
            "{}{} {}({}) {}",
            docs.dimmed(),
            "#define".yellow(),
            self.name.magenta(),
            self.inputs
                .iter()
                .map(|input| { scope.get_var(*input).input_text() })
                .collect::<Vec<_>>()
                .join(", "),
            self.expr.emit(scope)
        )
    }
}
