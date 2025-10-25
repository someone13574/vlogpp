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
            .map(|x| scope.new_var(&x.to_string(), false, false, None))
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
                variadicified_vars: None,
                calling_split: None,
                doc_name: None,
            })
        } else {
            let name = scope.get_alias(&format!("PASTE_{inputs}"), false);
            scope.new_macro(Macro {
                scope_id: scope.id,
                name,
                expr: Expr::List(var_exprs, "##"),
                inputs: vars,
                variadicified_vars: None,
                calling_split: None,
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
            let variadic = scope.new_var("variadic", false, true, None);
            scope.new_macro(Macro {
                scope_id: scope.id,
                name: scope.get_alias("EVAL0", false),
                expr: Expr::Var(variadic),
                inputs: vec![variadic],
                variadicified_vars: None,
                calling_split: None,
                doc_name: None,
            })
        } else {
            let prev = Box::new(Expr::Macro(Registry::eval_multiplier(
                global_scope,
                idx - 1,
            )));
            let mut scope = global_scope.new_scope();
            let variadic = scope.new_var("variadic", false, true, None);
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
                variadicified_vars: None,
                calling_split: None,
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

        let variadic = scope.new_var("variadic", false, true, None);
        let empty = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias("EMPTY", false),
            expr: Expr::List(Vec::new(), ", "),
            inputs: vec![variadic],
            variadicified_vars: None,
            calling_split: None,
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

        let var = scope.new_var("x", false, false, None);
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
            variadicified_vars: None,
            calling_split: None,
            doc_name: None,
        });

        let variadic = scope.new_var("variadic", false, true, None);
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
            variadicified_vars: None,
            calling_split: None,
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
        let variadic = scope.new_var("variadic", false, true, None);
        let expand = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias("EXPAND", false),
            expr: Expr::Var(variadic),
            inputs: vec![variadic],
            variadicified_vars: None,
            calling_split: None,
            doc_name: None,
        });
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

        let var = scope.new_var("cont", false, false, None);
        let id = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias("IF", false),
            expr: Expr::Call {
                r#macro: Box::new(Expr::Macro(paste)),
                args: vec![Expr::Text(format!("{prefix}{PREFIX_SEP}")), Expr::Var(var)],
            },
            inputs: vec![var],
            variadicified_vars: None,
            calling_split: None,
            doc_name: None,
        });

        scope.registry_mut().if_macro = Some(id);
        id
    }

    pub fn repeat_macro(global_scope: &mut GlobalScope, macro_id: MacroID, cont_var: &str) {
        let r#macro = global_scope.get_macro(macro_id).clone();
        let scope_id = r#macro.scope_id;
        let macro_name = format!("REPEAT_{}", &r#macro.name);

        let obstruct_macro = Registry::obstruct_macro(global_scope);
        let if_macro = Registry::if_macro(global_scope);

        let mut scope = global_scope.get_mut_scope(scope_id);

        let mut repeat_inputs = Vec::new();
        let mut cont_var_id = None;
        for output in scope.local().output_names.as_ref().unwrap().clone() {
            if let Some(var) = scope.scope().local().input_map.get(&format!("{output}.i")) {
                repeat_inputs.push(*var);
            } else {
                let var = scope.new_var(&output, false, false, None);
                repeat_inputs.push(var);

                if &output == cont_var {
                    cont_var_id = Some(var);
                }
            }
        }

        let mut call_inputs = Vec::new();
        let mut passthrough = Vec::new();
        for &input in &r#macro.inputs {
            call_inputs.push(Expr::Var(input));
            if !repeat_inputs.contains(&input) {
                repeat_inputs.push(input);
                passthrough.push(Expr::Var(input));
            }
        }

        let indirect_macro = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias(&format!("{macro_name}_INDIRECT"), false),
            expr: Expr::List(Vec::new(), ""),
            inputs: Vec::new(),
            variadicified_vars: None,
            calling_split: None,
            doc_name: None,
        });

        let repeat_macro = scope.new_macro(Macro {
            scope_id: scope.id,
            name: scope.get_alias(&macro_name, false),
            expr: Expr::Call {
                r#macro: Box::new(Expr::Call {
                    r#macro: Box::new(Expr::Macro(if_macro)),
                    args: vec![Expr::Var(cont_var_id.unwrap())],
                }),
                args: vec![Expr::Call {
                    r#macro: Box::new(Expr::Call {
                        r#macro: Box::new(Expr::Call {
                            r#macro: Box::new(Expr::Macro(obstruct_macro)),
                            args: vec![Expr::Macro(indirect_macro)],
                        }),
                        args: vec![],
                    }),
                    args: std::iter::once(Expr::Call {
                        r#macro: Box::new(Expr::Macro(macro_id)),
                        args: call_inputs,
                    })
                    .chain(passthrough.into_iter())
                    .collect(),
                }],
            },
            inputs: repeat_inputs,
            variadicified_vars: None,
            calling_split: None,
            doc_name: None,
        });

        scope.get_mut_macro(indirect_macro).expr = Expr::Macro(repeat_macro);
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
            .register_lut(Lut::not())
            .register_lut(Lut::or())
            .register_lut(Lut::and())
            .register_lut(Lut::xor())
            .register_lut(Lut::mux())
    }
}
