use std::collections::HashMap;
use std::fmt::{self, Display};

use ordermap::OrderMap;

use crate::expr::{Expr, ExprContent, ExprID, Var, VarID};
use crate::local_scope::{LocalScope, LocalScopeID};
use crate::r#macro::{Macro, MacroID};
use crate::module::create_module;
use crate::yosys::Yosys;

pub struct GlobalScope {
    pub yosys: Yosys,

    next_macro_id: MacroID,
    next_scope_id: LocalScopeID,

    pub defines: OrderMap<String, String>,
    pub macros: OrderMap<MacroID, Macro>,
    pub scopes: HashMap<LocalScopeID, LocalScope>,

    pub modules: HashMap<String, MacroID>,
    pub paste_macros: HashMap<(usize, bool), MacroID>,
}

impl GlobalScope {
    pub fn new(yosys: Yosys) -> Self {
        Self {
            yosys,
            next_macro_id: MacroID(0),
            next_scope_id: LocalScopeID(0),
            defines: OrderMap::new(),
            macros: OrderMap::new(),
            scopes: HashMap::new(),
            modules: HashMap::new(),
            paste_macros: HashMap::new(),
        }
    }
}

impl GlobalScope {
    pub fn get_scope(&self, scope_id: LocalScopeID) -> &LocalScope {
        self.scopes.get(&scope_id).unwrap()
    }

    pub fn get_macro_scope(&self, macro_id: MacroID) -> &LocalScope {
        self.scopes
            .get(&self.macros.get(&macro_id).unwrap().scope_id)
            .unwrap()
    }

    pub fn get_var(&self, var_id: VarID, scope_id: LocalScopeID) -> &Var {
        self.get_scope(scope_id).vars.get(&var_id).unwrap()
    }

    pub fn get_expr(&self, expr_id: ExprID, scope_id: LocalScopeID) -> &Expr {
        self.get_scope(scope_id).exprs.get(&expr_id).unwrap()
    }
}

impl GlobalScope {
    pub fn get_mut_scope(&mut self, scope_id: LocalScopeID) -> &mut LocalScope {
        self.scopes.get_mut(&scope_id).unwrap()
    }
}

impl GlobalScope {
    pub fn new_local_scope(&mut self) -> LocalScopeID {
        let id = self.next_scope_id;
        self.next_scope_id.0 += 1;

        self.scopes.insert(
            id,
            LocalScope {
                next_var_id: VarID(0),
                next_expr_id: ExprID(0),
                vars: HashMap::new(),
                exprs: HashMap::new(),
                input_map: HashMap::new(),
                output_names: None,
            },
        );
        id
    }

    pub fn get_macro_prefix(&self, name: &str) -> String {
        let mut prefix;
        let mut suffix = None;

        while {
            prefix = if let Some(suffix) = suffix {
                format!("{}__{suffix}", preprocess_macro_name(name))
            } else {
                preprocess_macro_name(name)
            };

            self.macros
                .values()
                .map(|r#macro| &r#macro.name)
                .chain(self.defines.values())
                .any(|name| name.starts_with(&prefix))
        } {
            suffix = Some(suffix.map_or(0, |x| x + 1));
        }

        prefix
    }

    pub fn get_module(&mut self, name: &str) -> Option<MacroID> {
        if let Some(id) = self.modules.get(name) {
            Some(*id)
        } else {
            let Some(module) = self.yosys.modules.get(name).cloned() else {
                return None;
            };

            Some(create_module(name, &module, self))
        }
    }

    pub fn new_macro(
        &mut self,
        name: &str,
        body: ExprID,
        inputs: Vec<VarID>,
        scope_id: LocalScopeID,
    ) -> MacroID {
        let mut alias;
        let mut suffix = None;

        while {
            alias = if let Some(suffix) = suffix {
                format!("{}__{suffix}", preprocess_macro_name(name))
            } else {
                preprocess_macro_name(name)
            };

            self.macros
                .values()
                .map(|r#macro| &r#macro.name)
                .chain(self.defines.values())
                .any(|name| *name == alias)
        } {
            suffix = Some(suffix.map_or(0, |x| x + 1));
        }

        let id = self.next_macro_id;
        self.next_macro_id.0 += 1;

        self.macros.insert(
            id,
            Macro {
                id,
                scope_id,
                name: alias,
                body,
                inputs,
            },
        );
        id
    }
}

impl GlobalScope {
    pub fn paste_macro(&mut self, num_inputs: usize, expand: bool) -> MacroID {
        assert!(num_inputs > 1 && num_inputs <= 26);

        if let Some(id) = self.paste_macros.get(&(num_inputs, expand)) {
            return *id;
        }

        let scope_id = self.new_local_scope();
        let vars = ('a'..='z')
            .into_iter()
            .take(num_inputs)
            .map(|x| self.get_mut_scope(scope_id).new_var(&x.to_string(), false))
            .collect::<Vec<_>>();
        let (content, wrapper) = if expand {
            let exprs = vars
                .iter()
                .map(|var| {
                    self.get_mut_scope(scope_id)
                        .new_expr(ExprContent::Var(*var), None)
                })
                .collect::<Vec<_>>();
            (
                ExprContent::List(exprs),
                Some(self.paste_macro(num_inputs, false)),
            )
        } else {
            (ExprContent::Concat(vars.clone()), None)
        };

        let body = self.get_mut_scope(scope_id).new_expr(content, wrapper);
        let macro_id = self.new_macro(
            &format!("PASTE_{}{num_inputs}", if expand { "EXPAND_" } else { "" }),
            body,
            vars,
            scope_id,
        );

        self.paste_macros.insert((num_inputs, expand), macro_id);
        macro_id
    }
}

impl Display for GlobalScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.defines.iter() {
            writeln!(f, "#define {key} {value}")?;
        }

        for r#macro in self.macros.values() {
            writeln!(f, "{}", r#macro.emit(self))?;
        }

        Ok(())
    }
}

fn preprocess_macro_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .map(|c| c.to_ascii_uppercase())
        .collect::<String>()
}
