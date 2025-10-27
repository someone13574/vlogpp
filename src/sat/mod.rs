use std::collections::HashMap;

use rustsat::instances::SatInstance;
use rustsat::solvers::Solve;
use rustsat_glucose::simp::Glucose;

use crate::netlist::Netlist;
use crate::sat::eval::eval_module;

mod eval;
mod lut;
mod name;

pub struct Combinational {
    module: String,
    netlist: Netlist,
    constraints: Vec<HashMap<String, bool>>,
}

impl Combinational {
    pub fn new(netlist: Netlist, module: Option<&str>) -> Self {
        let module = if let Some(module) = module {
            assert!(netlist.modules.contains_key(module));
            module.to_string()
        } else {
            netlist
                .modules
                .iter()
                .find(|(_, module)| {
                    module
                        .attributes
                        .get("top")
                        .is_some_and(|top| u32::from_str_radix(top, 2).unwrap() != 0)
                })
                .expect("No modules with `top` attribute")
                .0
                .clone()
        };

        Self {
            module,
            netlist,
            constraints: Vec::new(),
        }
    }

    pub fn generate(&mut self) {
        let module = self.netlist.modules.get(&self.module).unwrap();

        for _ in 0..5 {
            let mut constaints = HashMap::new();
            for (input, _) in module.input_ports() {
                constaints.insert(input.clone(), rand::random::<bool>());
            }

            constaints.extend(eval_module(&self.netlist, &self.module, constaints.clone()));
            self.constraints.push(constaints);
        }

        println!("{:?}", self.constraints);

        // self.generate_candidate();
    }

    pub fn generate_candidate(&self) {
        // let mut instance: SatInstance = SatInstance::new();

        // let mut solver = Glucose::default();
        // solver.add_cnf(instance.sanitize().into_cnf().0).unwrap();
        // solver.solve().unwrap();

        // let solution = solver.full_solution().unwrap();
    }
}
