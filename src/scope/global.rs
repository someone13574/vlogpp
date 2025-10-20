use std::collections::HashMap;
use std::fmt::{self, Display};

use crate::r#macro::{Macro, MacroID};
use crate::registry::Registry;
use crate::scope::local::{LocalScope, LocalScopeID};
use crate::scope::{MutScope, Scope};
use crate::{Map, Set};

pub struct GlobalScope {
    registry: Registry,

    next_scope_id: LocalScopeID,
    next_macro_id: MacroID,

    pub scopes: HashMap<LocalScopeID, LocalScope>,
    macros: Map<MacroID, Macro>,
    defines: Map<String, String>,
}

impl GlobalScope {
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            next_scope_id: LocalScopeID(0),
            next_macro_id: MacroID(0),
            scopes: HashMap::new(),
            macros: Map::new(),
            defines: Map::new(),
        }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }

    pub fn new_scope<'a>(&'a mut self) -> MutScope<'a> {
        let id = self.next_scope_id;
        self.next_scope_id.0 += 1;

        self.scopes.insert(id, LocalScope::new());
        self.get_mut_scope(id)
    }

    pub fn get_scope<'a>(&'a self, id: LocalScopeID) -> Scope<'a> {
        Scope {
            global: self,
            local: id,
        }
    }

    pub fn get_mut_scope<'a>(&'a mut self, id: LocalScopeID) -> MutScope<'a> {
        MutScope {
            global: self,
            local: id,
        }
    }

    pub fn new_macro(&mut self, r#macro: Macro) -> MacroID {
        assert!(self.name_available(&r#macro.name, false));

        let id = self.next_macro_id;
        self.next_macro_id.0 += 1;

        self.macros.insert(id, r#macro);
        id
    }

    pub fn get_macro(&self, id: MacroID) -> &Macro {
        self.macros.get(&id).unwrap()
    }

    pub fn get_mut_macro(&mut self, id: MacroID) -> &mut Macro {
        self.macros.get_mut(&id).unwrap()
    }

    pub fn define(&mut self, key: String, value: String) {
        assert!(self.name_available(&key, false));
        self.defines.insert(key, value);
    }

    #[cfg(not(feature = "obfuscate"))]
    pub fn get_alias(&self, name: &str, prefix: bool) -> String {
        let mut alias;
        let mut suffix = None;

        while {
            alias = if let Some(suffix) = suffix {
                format!("{}_{suffix}", preprocess_macro_name(name))
            } else {
                preprocess_macro_name(name)
            };

            !self.name_available(&alias, prefix)
        } {
            suffix = Some(suffix.map_or(0, |x| x + 1));
        }

        alias
    }

    #[cfg(feature = "obfuscate")]
    pub fn get_alias(&self, _name: &str, prefix: bool) -> String {
        const TRIES_PER_LENGTH: usize = 256;

        use rand::rngs::SmallRng;
        use rand::{Rng, SeedableRng};

        let mut rng = SmallRng::from_os_rng();

        for len in 1.. {
            for _ in 0..TRIES_PER_LENGTH {
                let mut alias = String::with_capacity(len);
                let idx = rng.random_range(0..26);
                alias.push(('A'..='Z').nth(idx).unwrap());
                for _ in 1..len {
                    let idx = rng.random_range(0..36);
                    alias.push(('A'..='Z').chain('0'..='9').nth(idx).unwrap());
                }

                if self.name_available(&alias, prefix) {
                    return alias;
                }
            }
        }

        unreachable!()
    }

    fn name_available(&self, name: &str, prefix: bool) -> bool {
        !self
            .macros
            .values()
            .map(|r#macro| &r#macro.name)
            .chain(self.defines.keys())
            .any(|existing| {
                if prefix {
                    existing.starts_with(name)
                } else {
                    existing == name
                }
            })
    }
}

impl Display for GlobalScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lines = Set::new();

        for (key, value) in self.defines.iter() {
            lines.insert(format!("#define {key} {value}"));
        }

        for r#macro in self.macros.values() {
            lines.insert(format!(
                "{}",
                if cfg!(feature = "obfuscate") {
                    r#macro.emit(self).replace(", ", ",")
                } else {
                    r#macro.emit(self)
                }
            ));
        }

        for line in lines {
            writeln!(f, "{line}")?;
        }

        Ok(())
    }
}

#[cfg(not(feature = "obfuscate"))]
fn preprocess_macro_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .map(|c| c.to_ascii_uppercase())
        .collect::<String>()
}
