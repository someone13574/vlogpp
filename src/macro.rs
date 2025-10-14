use crate::expr::ExprID;
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
}

impl Macro {
    pub fn emit(&self, global_scope: &GlobalScope) -> String {
        let local_scope = global_scope.get_scope(self.scope_id);
        format!(
            "{}(...) {}",
            self.name,
            local_scope
                .get_expr(self.body)
                .emit(global_scope, local_scope)
        )
    }
}
