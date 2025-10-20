use std::collections::HashMap;

use crate::expr::{Var, VarID};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct LocalScopeID(pub usize);

pub struct LocalScope {
    next_var_id: VarID,
    vars: HashMap<VarID, Var>,

    pub input_map: HashMap<String, VarID>,
    pub output_names: Option<Vec<String>>,
}

impl LocalScope {
    pub fn new() -> Self {
        Self {
            next_var_id: VarID(0),
            vars: HashMap::new(),
            input_map: HashMap::new(),
            output_names: None,
        }
    }

    #[cfg(not(feature = "obfuscate"))]
    pub fn new_var(&mut self, name: &str, map_input: bool) -> VarID {
        let mut alias;
        let mut suffix = None;

        while {
            alias = if let Some(suffix) = suffix {
                format!("{}{suffix}", preprocess_var_name(name))
            } else {
                preprocess_var_name(name)
            };

            self.vars.values().any(|existing| existing.name == alias)
        } {
            suffix = Some(suffix.map_or(0, |x| x + 1));
        }

        let id = self.next_var_id;
        self.next_var_id.0 += 1;
        self.vars.insert(id, Var { id, name: alias });

        if map_input {
            assert!(self.input_map.insert(name.to_string(), id).is_none());
        }

        id
    }

    #[cfg(feature = "obfuscate")]
    pub fn new_var(&mut self, name: &str, map_input: bool) -> VarID {
        const TRIES_PER_LENGTH: usize = 256;

        use rand::rngs::SmallRng;
        use rand::{Rng, SeedableRng};

        let mut rng = SmallRng::from_os_rng();

        for len in 1.. {
            for _ in 0..TRIES_PER_LENGTH {
                let mut alias = String::with_capacity(len);
                let idx = rng.random_range(0..26);
                alias.push(('a'..='z').nth(idx).unwrap());
                for _ in 1..len {
                    let idx = rng.random_range(0..36);
                    alias.push(('a'..='z').chain('0'..='9').nth(idx).unwrap());
                }

                if !self.vars.values().any(|existing| existing.name == alias) {
                    let id = self.next_var_id;
                    self.next_var_id.0 += 1;
                    self.vars.insert(id, Var { id, name: alias });

                    if map_input {
                        assert!(self.input_map.insert(name.to_string(), id).is_none());
                    }

                    return id;
                }
            }
        }

        unreachable!()
    }

    pub fn set_outputs(&mut self, outputs: Vec<String>) {
        self.output_names = Some(outputs);
    }

    pub fn get_var(&self, id: VarID) -> &Var {
        self.vars.get(&id).unwrap()
    }
}

impl Default for LocalScope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "obfuscate"))]
fn preprocess_var_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .map(|c| c.to_ascii_lowercase())
        .collect::<String>()
}
