// use crate::expr::{ExprID, VarID};
// use crate::global_scope::GlobalScope;
// use crate::local_scope::LocalScopeID;

use crate::expr::{Expr, VarID};
use crate::scope::global::GlobalScope;
use crate::scope::local::LocalScopeID;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct MacroID(pub usize);

pub struct Macro {
    pub scope_id: LocalScopeID,

    pub name: String,
    pub expr: Expr,

    pub inputs: Vec<VarID>,
    pub output_to_input: Option<usize>,
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

    pub fn emit(&self, global_scope: &GlobalScope) -> String {
        let scope = global_scope.get_scope(self.scope_id);
        format!(
            "#define {}({}) {}",
            self.name,
            self.inputs
                .iter()
                .map(|input| scope.get_var(*input).name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            self.expr.emit(scope)
        )
    }
}
