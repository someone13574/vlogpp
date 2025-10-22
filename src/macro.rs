use std::collections::HashMap;

use colored::Colorize;
use strip_ansi_escapes::strip_str;

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
    pub variadicified_vars: Option<Vec<VarID>>,
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

    pub fn variadicify_vars(&mut self, min_replace: usize) {
        assert_ne!(min_replace, 0);

        if self.variadicified_vars.is_some() {
            return;
        }

        let mut longest_span = None;
        for span in self.expr.var_spans() {
            if span.len()
                <= longest_span
                    .as_ref()
                    .map_or(min_replace - 1, |longest_span: &Vec<VarID>| {
                        longest_span.len()
                    })
            {
                continue;
            }

            if span.len() > self.inputs.len() {
                continue;
            }

            if span
                .iter()
                .zip(self.inputs.iter().skip(self.inputs.len() - span.len()))
                .any(|(a, b)| a != b)
            {
                continue;
            }

            if span.iter().any(|var| {
                self.expr
                    .vars()
                    .iter()
                    .filter(|expr_var| var == *expr_var)
                    .count()
                    != 1
            }) {
                continue;
            }

            longest_span = Some(span);
        }

        self.variadicified_vars = longest_span;
    }

    pub fn sort_passthrough_vars(id: MacroID, scope: &mut MutScope) {
        let caller_vars = scope
            .get_macro(id)
            .calling_split
            .map(|caller| scope.get_macro(caller).inputs.clone());

        // Get permutation indices per block (single var or bundle of vars)
        let mut input_blocks_perm = Vec::new();
        let mut bundle_blocks = HashMap::new();
        let mut total_idx = 0;
        let mut last_bundle = None;

        for &input in &scope.get_macro(id).inputs {
            let block_idx = input_blocks_perm.len();

            if let Some(bundle_var) = scope.get_var(input).bundle_id {
                if caller_vars
                    .as_ref()
                    .is_some_and(|caller_vars| caller_vars.contains(&bundle_var))
                    && bundle_blocks.get(&bundle_var).is_none()
                {
                    bundle_blocks.insert(bundle_var, block_idx);
                    input_blocks_perm.push((block_idx, total_idx, Vec::new(), 0.0));
                }

                if let Some(&bundle_block_idx) = bundle_blocks.get(&bundle_var) {
                    let (_idx, _total_idx, vars, max_score): &mut (usize, usize, Vec<VarID>, f64) =
                        input_blocks_perm.get_mut(bundle_block_idx).unwrap();
                    assert!(!vars.contains(&input));
                    assert!(last_bundle == Some(bundle_var) || vars.is_empty());

                    vars.push(input);
                    *max_score = max_score.max(
                        scope
                            .get_macro(id)
                            .var_passthrough_score(input, &scope.scope())
                            .0,
                    );

                    total_idx += 1;
                    last_bundle = Some(bundle_var);
                    continue;
                }
            }

            input_blocks_perm.push((
                block_idx,
                total_idx,
                vec![input],
                scope
                    .get_macro(id)
                    .var_passthrough_score(input, &scope.scope())
                    .0,
            ));
            total_idx += 1;
        }

        input_blocks_perm.sort_by(|(_, _, _, a), (_, _, _, b)| a.partial_cmp(b).unwrap());

        // Get input permuation
        let input_perm = input_blocks_perm
            .iter()
            .flat_map(|(_, total_idx, vars, _)| {
                vars.iter()
                    .enumerate()
                    .map(move |(idx_in_block, _var)| total_idx + idx_in_block)
            })
            .collect::<Vec<_>>();

        // Reorder inputs
        let inputs = &mut scope.get_mut_macro(id).inputs;
        *inputs = input_perm
            .iter()
            .map(|idx| *inputs.get(*idx).unwrap())
            .collect();

        // Reorder caller exprs
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

            *args = input_blocks_perm
                .iter()
                .map(|(idx, _, _, _)| args.get(*idx).unwrap().clone())
                .collect();
        }
    }

    fn var_passthrough_score(&self, id: VarID, scope: &Scope) -> (f64, f64) {
        let (caller_score, caller_max_score) = if let Some(caller) = self
            .calling_split
            .iter()
            .find(|caller| scope.get_macro(**caller).scope_id == scope.id)
        {
            scope.get_macro(*caller).var_passthrough_score(id, scope)
        } else {
            (0.0, 1.0)
        };

        let max_expr_score = self.expr.vars().len() as f64;
        let expr_score = self
            .expr
            .vars()
            .iter()
            .position(|var| *var == id)
            .unwrap_or_default() as f64;
        let indiv_score = if self.is_passthrough_var(id) {
            max_expr_score + expr_score
        } else {
            0.0
        };
        let max_indiv_score = max_expr_score * 2.0;

        let out = (
            caller_score + indiv_score * caller_max_score,
            caller_max_score + max_indiv_score * caller_max_score,
        );
        assert!(out.0.is_finite());
        assert!(out.1.is_finite());
        out
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

        let mut inputs = self
            .inputs
            .iter()
            .map(|input| scope.get_var(*input).input_text())
            .collect::<Vec<_>>()
            .join(", ");
        let mut expr_text = self.expr.emit(scope);

        if let Some(variadicified_vars) = &self.variadicified_vars {
            let variadic_inputs = self
                .inputs
                .iter()
                .take(self.inputs.len() - variadicified_vars.len())
                .map(|var| scope.get_var(*var).input_text())
                .chain(std::iter::once("..."))
                .collect::<Vec<_>>()
                .join(", ");

            let span = variadicified_vars
                .iter()
                .map(|var| scope.get_var(*var).expr_text())
                .collect::<Vec<_>>()
                .join(", ");
            let variadic_expr_text =
                expr_text.replace(&span, "__VA_ARGS__".magenta().to_string().as_str());

            if cfg!(not(feature = "obfuscate"))
                || strip_str(&variadic_inputs).len() + strip_str(&variadic_expr_text).len()
                    < strip_str(&inputs).len() + strip_str(&expr_text).len()
            {
                inputs = variadic_inputs;
                expr_text = variadic_expr_text;
            }
        }

        format!(
            "{}{} {}({}) {}",
            docs.dimmed(),
            "#define".yellow(),
            self.name.magenta(),
            inputs,
            expr_text
        )
    }
}
