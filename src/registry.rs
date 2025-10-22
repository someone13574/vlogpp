use std::collections::HashMap;

use crate::PREFIX_SEP;
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
    eval_macros: Vec<MacroID>,
    empty_macro: Option<MacroID>,
    obstruct_macro: Option<MacroID>,
    if_macro: Option<MacroID>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            luts: HashMap::new(),
            modules: HashMap::new(),
            module_macros: HashMap::new(),
            paste_macros: HashMap::new(),
            eval_macros: Vec::new(),
            empty_macro: None,
            obstruct_macro: None,
            if_macro: None,
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

    pub fn top_modules(global_scope: &mut GlobalScope) -> Vec<MacroID> {
        let mut macros = Vec::new();

        for name in global_scope
            .registry()
            .modules
            .iter()
            .filter(|(_, module)| {
                module
                    .attributes
                    .get("top")
                    .is_some_and(|value| u32::from_str_radix(value, 2).unwrap() != 0)
            })
            .map(|(name, _)| name.to_string())
            .collect::<Vec<_>>()
        {
            macros.push(Registry::module(global_scope, &name).unwrap());
        }

        macros
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
            .map(|x| scope.new_var(&x.to_string(), false, false))
            .collect::<Vec<_>>();
        let var_exprs = vars.iter().map(|&var| Expr::Var(var)).collect::<Vec<_>>();

        let macro_id = if expand {
            let raw_paste_macro = Registry::paste_macro(scope.global, inputs, false);
            let name = scope.get_alias(&format!("PASTE_EXPAND_{inputs}"), false);
            scope.new_macro(Macro {
                scope_id: scope.id,
                name,
                expr: Expr::Call {
                    r#macro: Box::new(Expr::Macro(raw_paste_macro)),
                    args: var_exprs,
                },
                inputs: vars,
                calling_split: None,
                output_to_input: None,
                doc_name: None,
            })
        } else {
            let name = scope.get_alias(&format!("PASTE_{inputs}"), false);
            scope.new_macro(Macro {
                scope_id: scope.id,
                name,
                expr: Expr::List(var_exprs, "##"),
                inputs: vars,
                calling_split: None,
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

    pub fn eval_multiplier(global_scope: &mut GlobalScope, idx: usize) -> MacroID {
        if let Some(id) = global_scope.registry().eval_macros.get(idx) {
            return *id;
        }

        let id = if idx == 0 {
            let mut scope = global_scope.new_scope();
            let variadic = scope.new_var("variadic", false, true);
            scope.new_macro(Macro {
                scope_id: scope.id,
                name: scope.get_alias("EVAL0", false),
                expr: Expr::Var(variadic),
                inputs: vec![variadic],
                calling_split: None,
                output_to_input: None,
                doc_name: None,
            })
        } else {
            let prev = Box::new(Expr::Macro(Registry::eval_multiplier(
                global_scope,
                idx - 1,
            )));
            let mut scope = global_scope.new_scope();
            let variadic = scope.new_var("variadic", false, true);
            scope.new_macro(Macro {
                scope_id: scope.id,
                name: scope.get_alias(&format!("EVAL{idx}"), false),
                expr: Expr::Call {
                    r#macro: prev.clone(),
                    args: vec![Expr::Call {
                        r#macro: prev.clone(),
                        args: vec![Expr::Call {
                            r#macro: prev.clone(),
                            args: vec![Expr::Call {
                                r#macro: prev,
                                args: vec![Expr::Var(variadic)],
                            }],
                        }],
                    }],
                },
                inputs: vec![variadic],
                calling_split: None,
                output_to_input: None,
                doc_name: None,
            })
        };

        global_scope.registry_mut().eval_macros.push(id);
        id
    }

    pub fn empty_macro(global_scope: &mut GlobalScope) -> MacroID {
        if let Some(empty) = global_scope.registry().empty_macro {
            return empty;
        }

        let mut scope = global_scope.new_scope();

        let variadic = scope.new_var("variadic", false, true);
        let empty = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias("EMPTY", false),
            expr: Expr::List(Vec::new(), ", "),
            inputs: vec![variadic],
            calling_split: None,
            output_to_input: None,
            doc_name: None,
        });

        global_scope.registry_mut().empty_macro = Some(empty);
        empty
    }

    pub fn obstruct_macro(global_scope: &mut GlobalScope) -> MacroID {
        if let Some(obstruct) = global_scope.registry().obstruct_macro {
            return obstruct;
        }

        let mut scope = global_scope.new_scope();

        let empty = Registry::empty_macro(scope.global);

        let var = scope.new_var("x", false, false);
        let defer = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias("DEFER", false),
            expr: Expr::List(
                vec![
                    Expr::Var(var),
                    Expr::Call {
                        r#macro: Box::new(Expr::Macro(empty)),
                        args: Vec::new(),
                    },
                ],
                " ",
            ),
            inputs: vec![var],
            calling_split: None,
            output_to_input: None,
            doc_name: None,
        });

        let variadic = scope.new_var("variadic", false, true);
        let obstruct = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias("OBSTRUCT", false),
            expr: Expr::List(
                vec![
                    Expr::Var(variadic),
                    Expr::Call {
                        r#macro: Box::new(Expr::Call {
                            r#macro: Box::new(Expr::Macro(defer)),
                            args: vec![Expr::Macro(empty)],
                        }),
                        args: Vec::new(),
                    },
                ],
                " ",
            ),
            inputs: vec![variadic],
            calling_split: None,
            output_to_input: None,
            doc_name: None,
        });

        scope.registry_mut().obstruct_macro = Some(obstruct);
        obstruct
    }

    pub fn if_macro(global_scope: &mut GlobalScope) -> MacroID {
        if let Some(id) = global_scope.registry().if_macro {
            return id;
        }

        let mut scope = global_scope.new_scope();
        let expand = Registry::eval_multiplier(scope.global, 0);
        let eat = Registry::empty_macro(scope.global);

        let prefix = scope.get_alias("IF", true);
        scope.define(
            format!("{prefix}{PREFIX_SEP}0"),
            scope.get_macro(eat).name.clone(),
        );
        scope.define(
            format!("{prefix}{PREFIX_SEP}1"),
            scope.get_macro(expand).name.clone(),
        );

        let paste = Registry::paste_macro(scope.global, 2, true);

        let var = scope.new_var("cont", false, false);
        let id = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias("IF", false),
            expr: Expr::Call {
                r#macro: Box::new(Expr::Macro(paste)),
                args: vec![Expr::Text(format!("{prefix}{PREFIX_SEP}")), Expr::Var(var)],
            },
            inputs: vec![var],
            calling_split: None,
            output_to_input: None,
            doc_name: None,
        });

        scope.registry_mut().if_macro = Some(id);
        id
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
