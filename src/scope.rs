use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use ordermap::OrderMap;

use crate::{
    expr::{Expr, ExprContent, ExprID, Var, VarID},
    r#macro::{Macro, MacroID},
    yosys::Yosys,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct LocalScopeID(pub usize);

pub struct GlobalScope {
    pub verilog: Yosys,

    next_macro_id: MacroID,
    next_scope_id: LocalScopeID,
    pub macros: OrderMap<MacroID, Macro>,
    pub scopes: HashMap<LocalScopeID, LocalScope>,

    pub modules: HashMap<String, MacroID>,
    pub paste_macros: HashMap<usize, MacroID>,
    pub expand_paste_macros: HashMap<usize, MacroID>,
}

impl GlobalScope {
    pub fn new(verilog: Yosys) -> Self {
        Self {
            verilog,
            next_macro_id: MacroID(0),
            next_scope_id: LocalScopeID(0),
            macros: OrderMap::new(),
            scopes: HashMap::new(),
            modules: HashMap::new(),
            paste_macros: HashMap::new(),
            expand_paste_macros: HashMap::new(),
        }
    }

    pub fn get_root_macro(&mut self) -> Option<&Macro> {
        for (name, module) in self.verilog.modules.clone() {
            if module
                .attributes
                .get("top")
                .is_some_and(|val| u32::from_str_radix(val, 2).unwrap() == 1)
            {
                return Some(self.get_module_macro(&name));
            }
        }

        None
    }

    pub fn get_module_macro(&mut self, name: &str) -> &Macro {
        if let Some(macro_id) = self.modules.get(name) {
            self.macros.get(macro_id).unwrap()
        } else {
            todo!()
        }
    }

    pub fn new_macro(&mut self, name: &str, expr: ExprID, scope_id: LocalScopeID) -> MacroID {
        let mut alias = name.to_uppercase();
        let mut suffix = None;

        let mut alias_attempt;
        while {
            alias_attempt = if let Some(suffix) = suffix {
                format!("{alias}__{suffix}")
            } else {
                alias.clone()
            };

            self.macros
                .values()
                .any(|r#macro| r#macro.name == alias_attempt)
        } {
            suffix = Some(suffix.map_or(0, |x| x + 1));
        }

        alias = alias_attempt;

        let id = self.next_macro_id;
        self.next_macro_id.0 += 1;

        self.macros.insert(
            id,
            Macro {
                id,
                scope_id,
                name: alias,
                body: expr,
            },
        );

        id
    }

    pub fn new_local_scope(&mut self) -> LocalScopeID {
        let id = self.next_scope_id;
        self.next_scope_id.0 += 1;

        self.scopes.insert(
            id,
            LocalScope {
                next_var_id: VarID(0),
                next_expr_id: ExprID(0),
                slices: Vec::new(),
                vars: HashMap::new(),
                exprs: HashMap::new(),
            },
        );
        id
    }

    pub fn get_paste_macro(&mut self, num_inputs: usize, expand: bool) -> MacroID {
        assert!(num_inputs > 1);

        if let Some(id) = if expand {
            &self.expand_paste_macros
        } else {
            &self.paste_macros
        }
        .get(&num_inputs)
        {
            return *id;
        }

        let scope_id = self.new_local_scope();
        let inputs = (0..num_inputs)
            .map(|idx| {
                self.scopes
                    .get_mut(&scope_id)
                    .unwrap()
                    .new_input(["a", "b", "c", "d", "e", "f"].get(idx).unwrap())
            })
            .collect::<Vec<_>>();

        let (content, wrapper) = if expand {
            let exprs = inputs
                .iter()
                .map(|input| {
                    self.scopes
                        .get_mut(&scope_id)
                        .unwrap()
                        .new_expr(ExprContent::Var(*input), None)
                })
                .collect::<Vec<_>>();

            (
                ExprContent::List(exprs),
                Some(self.get_paste_macro(num_inputs, false)),
            )
        } else {
            (ExprContent::Concat(inputs), None)
        };

        let expr = self
            .scopes
            .get_mut(&scope_id)
            .unwrap()
            .new_expr(content, wrapper);
        let id = self.new_macro(
            &format!("PASTE{}_{num_inputs}", if expand { "_EX" } else { "" }),
            expr,
            scope_id,
        );

        self.paste_macros.insert(num_inputs, id);
        id
    }
}

impl Display for GlobalScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r#macro in self.macros.values() {
            writeln!(f, "{}", r#macro.emit(self))?;
        }

        Ok(())
    }
}

pub struct LocalScope {
    next_var_id: VarID,
    next_expr_id: ExprID,

    pub slices: Vec<MacroID>,
    pub vars: HashMap<VarID, Var>,
    pub exprs: HashMap<ExprID, Expr>,
}

impl LocalScope {
    pub fn get_var(&self, id: &VarID) -> &Var {
        self.vars.get(id).unwrap()
    }

    pub fn get_expr(&self, id: &ExprID) -> &Expr {
        self.exprs.get(id).unwrap()
    }

    pub fn new_expr(&mut self, content: ExprContent, wrapper: Option<MacroID>) -> ExprID {
        let id = self.next_expr_id;
        self.next_expr_id.0 += 1;

        self.exprs.insert(
            id,
            Expr {
                id,
                wrapper,
                content,
            },
        );
        id
    }

    pub fn new_input(&mut self, name: &str) -> VarID {
        let mut alias = name.to_lowercase();
        let mut suffix = None;

        let mut alias_attempt;
        while {
            alias_attempt = if let Some(suffix) = suffix {
                format!("{alias}__{suffix}")
            } else {
                alias.clone()
            };

            self.vars.values().any(|var| var.name == alias_attempt)
        } {
            suffix = Some(suffix.map_or(0, |x| x + 1));
        }

        alias = alias_attempt;

        let id = self.next_var_id;
        self.next_var_id.0 += 1;

        self.vars.insert(id, Var { id, name: alias });
        id
    }
}
