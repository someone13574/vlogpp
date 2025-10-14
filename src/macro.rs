use crate::expr::{ExprID, VarID};
use crate::global_scope::GlobalScope;
use crate::local_scope::LocalScopeID;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct MacroID(pub usize);

#[derive(Clone)]
pub struct Macro {
    pub id: MacroID,
    pub scope_id: LocalScopeID,

    pub name: String,
    pub body: ExprID,
    pub inputs: Vec<VarID>,
}

impl Macro {
    pub fn input_position(&self, name: &str, global_scope: &GlobalScope) -> Option<usize> {
        let local_scope = global_scope.get_scope(self.scope_id);
        let var_id = local_scope.input_map.get(name).unwrap();
        self.inputs.iter().position(|input| input == var_id)
    }

    pub fn emit(&self, global_scope: &GlobalScope) -> String {
        let local_scope = global_scope.get_scope(self.scope_id);
        format!(
            "#define {}({}) {}",
            self.name,
            self.inputs
                .iter()
                .map(|input| format!("{}", local_scope.get_var(*input)))
                .collect::<Vec<_>>()
                .join(", "),
            local_scope
                .get_expr(self.body)
                .emit(global_scope, local_scope)
        )
    }
}
