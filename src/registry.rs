use std::collections::HashMap;

use crate::expr::Expr;
use crate::lut::Lut;
use crate::r#macro::{Macro, MacroID};
use crate::module::create_module;
use crate::netlist::{Module, Netlist};
use crate::scope::global::GlobalScope;

pub struct Registry {
    luts: HashMap<String, Lut>,
    modules: HashMap<String, Module>,

    module_macros: HashMap<String, MacroID>,
    paste_macros: HashMap<(usize, bool), MacroID>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            luts: HashMap::new(),
            modules: HashMap::new(),
            module_macros: HashMap::new(),
            paste_macros: HashMap::new(),
        }
    }

    pub fn add_netlist(mut self, netlist: Netlist) -> Self {
        for (name, module) in netlist.modules {
            assert!(self.name_available(&name));
            self.modules.insert(name, module);
        }

        self
    }

    pub fn register_lut(mut self, lut: Lut) -> Self {
        assert!(self.name_available(lut.name));
        self.luts.insert(lut.name.to_string(), lut);
        self
    }

    pub fn module(global_scope: &mut GlobalScope, name: &str) -> Option<MacroID> {
        if let Some(&id) = global_scope.registry_mut().module_macros.get(name) {
            return Some(id);
        }

        if let Some(lut) = global_scope.registry().luts.get(name).cloned() {
            let macro_id = lut.make_macro(global_scope);
            global_scope
                .registry_mut()
                .module_macros
                .insert(name.to_string(), macro_id);

            return Some(macro_id);
        }

        if let Some(module) = global_scope.registry().modules.get(name).cloned() {
            let macro_id = create_module(name, &module, global_scope);
            global_scope
                .registry_mut()
                .module_macros
                .insert(name.to_string(), macro_id);

            return Some(macro_id);
        }

        None
    }

    pub fn paste_macro(global_scope: &mut GlobalScope, inputs: usize, expand: bool) -> MacroID {
        assert!(inputs > 1);

        if let Some(macro_id) = global_scope
            .registry_mut()
            .paste_macros
            .get(&(inputs, expand))
        {
            return *macro_id;
        }

        let mut scope = global_scope.new_scope();
        let vars = ('a'..='z')
            .cycle()
            .take(inputs)
            .map(|x| scope.new_var(&x.to_string(), false))
            .collect::<Vec<_>>();
        let var_exprs = vars.iter().map(|&var| Expr::Var(var)).collect::<Vec<_>>();

        let macro_id = if expand {
            let raw_paste_macro = Registry::paste_macro(scope.global, inputs, false);
            let name = scope.get_alias(&format!("PASTE_EXPAND_{inputs}"), false);
            scope.new_macro(Macro {
                scope_id: scope.local,
                name,
                expr: Expr::Call {
                    r#macro: Box::new(Expr::Macro(raw_paste_macro)),
                    args: var_exprs,
                },
                inputs: vars,
                output_to_input: None,
                doc_name: None,
            })
        } else {
            let name = scope.get_alias(&format!("PASTE_{inputs}"), false);
            scope.new_macro(Macro {
                scope_id: scope.local,
                name,
                expr: Expr::Concat(var_exprs),
                inputs: vars,
                output_to_input: None,
                doc_name: None,
            })
        };

        global_scope
            .registry_mut()
            .paste_macros
            .insert((inputs, expand), macro_id);
        macro_id
    }

    pub fn top_module(global_scope: &mut GlobalScope) -> Option<MacroID> {
        if let Some(name) = global_scope
            .registry()
            .modules
            .iter()
            .find(|(_, module)| {
                module
                    .attributes
                    .get("top")
                    .is_some_and(|value| u32::from_str_radix(value, 2).unwrap() != 0)
            })
            .map(|(name, _)| name.to_string())
        {
            Registry::module(global_scope, &name)
        } else {
            None
        }
    }

    fn name_available(&self, name: &str) -> bool {
        !self
            .luts
            .keys()
            .chain(self.modules.keys())
            .any(|existing| existing == name)
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
