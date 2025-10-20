use delegate::delegate;

use crate::expr::{Var, VarID};
use crate::r#macro::{Macro, MacroID};
use crate::registry::Registry;
use crate::scope::global::GlobalScope;
use crate::scope::local::{LocalScope, LocalScopeID};

pub mod global;
pub mod local;

#[derive(Clone, Copy)]
pub struct Scope<'a> {
    pub global: &'a GlobalScope,
    pub id: LocalScopeID,
}

pub struct MutScope<'a> {
    pub global: &'a mut GlobalScope,
    pub id: LocalScopeID,
}

impl Scope<'_> {
    delegate! {
        to self.global {
            pub fn registry(&self) -> &Registry;
            pub fn get_macro(&self, id: MacroID) -> &Macro;
            pub fn get_alias(&self, name: &str, prefix: bool) -> String;
        }
    }

    delegate! {
        to self.global.scopes.get(&self.id).unwrap() {
            pub fn get_var(&self, id: VarID) -> &Var;
        }
    }

    pub fn local(&self) -> &LocalScope {
        self.global.scopes.get(&self.id).unwrap()
    }
}

impl<'a> MutScope<'a> {
    delegate! {
        to self.global {
            pub fn registry(&self) -> &Registry;
            pub fn registry_mut(&mut self) -> &mut Registry;
            pub fn new_macro(&mut self, r#macro: Macro) -> MacroID;
            pub fn get_macro(&self, id: MacroID) -> &Macro;
            pub fn get_mut_macro(&mut self, id: MacroID) -> &mut Macro;
            pub fn define(&mut self, key: String, value: String);
            pub fn get_alias(&self, name: &str, prefix: bool) -> String;
        }
    }

    delegate! {
        to self.global.scopes.get_mut(&self.id).unwrap() {
            pub fn new_var(&mut self, name: &str, map_input: bool, variadic: bool) -> VarID;
            pub fn set_outputs(&mut self, outputs: Vec<String>);
        }
    }

    delegate! {
        to self.global.scopes.get(&self.id).unwrap() {
            pub fn get_var(&self, id: VarID) -> &Var;
        }
    }

    pub fn scope(&'a self) -> Scope<'a> {
        Scope {
            global: self.global,
            id: self.id,
        }
    }

    pub fn local(&mut self) -> &mut LocalScope {
        self.global.scopes.get_mut(&self.id).unwrap()
    }
}
