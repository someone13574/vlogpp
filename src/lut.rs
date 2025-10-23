use std::iter::once;

use crate::PREFIX_SEP;
use crate::expr::Expr;
use crate::r#macro::{Macro, MacroID};
use crate::registry::Registry;
use crate::scope::global::GlobalScope;

#[derive(Clone)]
pub struct Lut {
    pub name: &'static str,

    pub input_names: &'static [&'static str],
    pub output_name: &'static str,

    pub outputs: &'static [bool],
}

impl Lut {
    pub fn make_macro(&self, global_scope: &mut GlobalScope) -> MacroID {
        assert!(!self.outputs.is_empty() && self.outputs.len().is_power_of_two());

        let num_inputs = self.outputs.len().ilog2() as usize;
        assert_eq!(num_inputs, self.input_names.len());

        let mut scope = global_scope.new_scope();
        let vars = self
            .input_names
            .iter()
            .map(|name| scope.new_var(name, true, false, None))
            .collect::<Vec<_>>();
        scope.local().output_names = Some(vec![self.output_name.to_string()]);

        let paste_macro = Registry::paste_macro(scope.global, num_inputs + 1, true);
        let prefix = scope.get_alias(self.name, true);

        for (idx, &output) in self.outputs.iter().enumerate() {
            scope.define(
                format!("{prefix}{PREFIX_SEP}{:0len$b}", idx, len = num_inputs),
                if output {
                    "1".to_string()
                } else {
                    "0".to_string()
                },
            );
        }

        scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias(self.name, false),
            expr: Expr::Call {
                r#macro: Box::new(Expr::Macro(paste_macro)),
                args: once(Expr::Text(format!("{prefix}{PREFIX_SEP}")))
                    .chain(vars.iter().map(|&var| Expr::Var(var)))
                    .collect(),
            },
            inputs: vars,
            variadicified_vars: None,
            calling_split: None,
            doc_name: Some(self.name.to_string()),
        })
    }

    pub fn not() -> Self {
        Self {
            name: "$_NOT_",
            input_names: &["A"],
            output_name: "Y",
            outputs: &[true, false],
        }
    }

    pub fn or() -> Self {
        Self {
            name: "$_OR_",
            input_names: &["A", "B"],
            output_name: "Y",
            outputs: &[false, true, true, true],
        }
    }

    pub fn and() -> Self {
        Self {
            name: "$_AND_",
            input_names: &["A", "B"],
            output_name: "Y",
            outputs: &[false, false, false, true],
        }
    }

    pub fn xor() -> Self {
        Self {
            name: "$_XOR_",
            input_names: &["A", "B"],
            output_name: "Y",
            outputs: &[false, true, true, false],
        }
    }

    pub fn dff_p() -> Self {
        Self {
            name: "$_DFF_P_",
            input_names: &["C", "D", "Q.i"],
            output_name: "Q",
            outputs: &[false, true, false, true, false, false, true, true],
        }
    }
}
