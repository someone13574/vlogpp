use std::collections::HashMap;

pub struct GlobalNames {
    splits: HashMap<String, Vec<String>>,
}

impl GlobalNames {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_split(&mut self, name: &str, split_idx: usize) -> String {
        if !self.splits.contains_key(name) {
            self.splits.insert(name.to_string(), Vec::new());
        }

        while self.splits.get(name).unwrap().len() < split_idx + 1 {
            let mut alias = if name.starts_with("_") {
                format!("U{}", name.to_uppercase())
            } else {
                name.to_uppercase()
            };

            if self.name_assigned(&alias) {
                let mut new_alias;
                let mut attempt = 0;
                while {
                    new_alias = format!("{alias}__{attempt}");
                    self.name_assigned(&new_alias)
                } {
                    attempt += 1;
                }

                alias = new_alias;
            }

            self.splits.get_mut(name).unwrap().push(alias);
        }

        self.splits
            .get(name)
            .unwrap()
            .get(split_idx)
            .unwrap()
            .clone()
    }

    fn name_assigned(&self, name: &str) -> bool {
        self.splits
            .values()
            .flatten()
            .any(|macro_name| macro_name == name)
    }
}

impl Default for GlobalNames {
    fn default() -> Self {
        Self {
            splits: HashMap::new(),
        }
    }
}

pub struct LocalNames {
    inputs: HashMap<String, String>,
    next_temp: usize,
}

impl LocalNames {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_input(&mut self, name: &str) -> String {
        let mut alias = if name.starts_with("_") {
            format!("u{}", name.to_lowercase().replace("[", "").replace("]", ""))
        } else {
            name.to_lowercase().replace("[", "").replace("]", "")
        };

        if self.name_assigned(&alias) {
            let mut new_alias;
            let mut attempt = 0;
            while {
                new_alias = format!("{alias}__{attempt}");
                self.name_assigned(&new_alias)
            } {
                attempt += 1;
            }

            alias = new_alias;
        }

        assert_eq!(self.inputs.insert(name.to_string(), alias.clone()), None);
        alias
    }

    fn name_assigned(&self, name: &str) -> bool {
        self.inputs.values().any(|input_name| input_name == name)
    }

    pub fn get_temp(&mut self) -> String {
        let name = format!("_t{}", self.next_temp);
        self.next_temp += 1;

        name
    }
}

impl Default for LocalNames {
    fn default() -> Self {
        Self {
            inputs: HashMap::new(),
            next_temp: 0,
        }
    }
}
