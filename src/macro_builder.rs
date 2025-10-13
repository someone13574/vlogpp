use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
};

use indexmap::IndexMap;

use crate::{
    gates::{
        AND_GATE_NAME, OR_GATE_NAME, XOR_GATE_NAME, implement_and, implement_or, implement_xor,
    },
    yosys::{Cell, Module, PortDirection},
};

#[derive(Debug, Clone)]
pub struct Macro {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub wrapper: Option<String>,
}

impl Display for Macro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(wrapper) = &self.wrapper {
            writeln!(
                f,
                "#define {}({}) {wrapper}({})",
                self.name,
                self.inputs.join(", "),
                self.outputs.join(", ")
            )
        } else {
            writeln!(
                f,
                "#define {}({}) {}",
                self.name,
                self.inputs.join(", "),
                self.outputs.join(", ")
            )
        }
    }
}

#[derive(Debug)]
pub struct ModuleMacro {
    pub cost: usize,
    pub input_map: HashMap<String, usize>,
    pub output_map: HashMap<String, usize>,
    pub splits: Vec<Macro>,
}

impl ModuleMacro {
    fn add_expr(
        &mut self,
        expr_idx: usize,
        add_split_idx: usize,
        exprs: &IndexMap<usize, ExpressionInfo>,
        split_var_exprs: &HashMap<String, usize>,
    ) {
        let expr = exprs.get(&expr_idx).unwrap();
        self.splits
            .get_mut(add_split_idx)
            .unwrap()
            .outputs
            .push(expr.text(false).unwrap().to_string());
        self.cost += expr.cost().unwrap();

        let skip = self.splits.len() - add_split_idx - 1;
        for var in expr.input_vars().unwrap() {
            let split_var_expr = split_var_exprs
                .get(var)
                .and_then(|expr_idx| exprs.get(expr_idx));
            let mut prev_input_idx = None;

            for (split_idx, split) in self.splits.iter_mut().enumerate().rev().skip(skip) {
                if let Some(split_var_expr) = split_var_expr {
                    if split_var_expr.split_idx(false).unwrap() == split_idx {
                        if prev_input_idx == Some(split.outputs.len()) {
                            self.add_expr(
                                *split_var_exprs.get(var).unwrap(),
                                split_idx,
                                exprs,
                                split_var_exprs,
                            );
                        }
                        break;
                    }
                }

                if prev_input_idx
                    .is_some_and(|prev_input_idx| prev_input_idx == split.outputs.len())
                {
                    split.outputs.push(var.to_string());
                }

                if let Some(pos) = split.inputs.iter().position(|x| x == var) {
                    prev_input_idx = Some(pos);
                } else {
                    prev_input_idx = Some(split.inputs.len());
                    split.inputs.push(var.to_string());
                }
            }
        }
    }
}

impl Display for ModuleMacro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut inputs: Vec<(&String, &usize)> = self.input_map.iter().collect();
        inputs.sort_by(|a, b| a.1.cmp(b.1).then_with(|| a.0.cmp(b.0))); // tie-breaker for determinism
        let inputs_str = inputs
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        let mut outputs: Vec<(&String, &usize)> = self.output_map.iter().collect();
        outputs.sort_by(|a, b| a.1.cmp(b.1).then_with(|| a.0.cmp(b.0)));
        let outputs_str = outputs
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(
            f,
            "/// Cost: {}, Inputs: {inputs_str}, Outputs: {outputs_str}",
            self.cost
        )?;
        for split in &self.splits {
            writeln!(f, "{split}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum ExpressionInfo<'a> {
    Implemented {
        text: String,
        cost: usize,
        split_idx: usize,
        split_var: Option<String>,
        input_vars: HashSet<String>,
    },
    Unimplemented {
        cell: &'a Cell,
    },
}

impl<'a> ExpressionInfo<'a> {
    fn text(&self, downstream: bool) -> Option<&str> {
        match self {
            Self::Implemented {
                text, split_var, ..
            } => {
                if downstream && split_var.is_some() {
                    Some(split_var.as_ref().unwrap().as_str())
                } else {
                    Some(text.as_str())
                }
            }
            _ => None,
        }
    }

    fn cost(&self) -> Option<usize> {
        match self {
            Self::Implemented { cost, .. } => Some(*cost),
            _ => None,
        }
    }

    fn split_idx(&self, downstream: bool) -> Option<usize> {
        match self {
            Self::Implemented {
                split_idx,
                split_var,
                ..
            } => Some(
                *split_idx
                    + if downstream && split_var.is_some() {
                        1
                    } else {
                        0
                    },
            ),
            _ => None,
        }
    }

    fn split_var(&self) -> Option<&str> {
        match self {
            Self::Implemented { split_var, .. } => split_var.as_ref().map(|x| x.as_str()),
            _ => None,
        }
    }

    fn input_vars(&self) -> Option<&HashSet<String>> {
        match self {
            Self::Implemented { input_vars, .. } => Some(input_vars),
            _ => None,
        }
    }

    fn cell(&self) -> Option<&Cell> {
        match self {
            Self::Unimplemented { cell } => Some(*cell),
            _ => None,
        }
    }
}

pub fn build_module_macro(name: &str, module: &Module) -> ModuleMacro {
    let port_names = assign_port_names(module);
    let mut split_var_exprs = HashMap::new();
    let mut exprs = assign_inputs(module, &port_names);
    collect_cell_outputs(module, &mut exprs);

    let consumer_counts = count_consumers(module);

    for idx in 0..exprs.len() {
        let (&expr_idx, _) = exprs.get_index(idx).unwrap();
        while let Some(implementable_expr_idx) = get_implementable_cell(expr_idx, &exprs) {
            implement_expr(
                implementable_expr_idx,
                &mut exprs,
                &consumer_counts,
                &mut split_var_exprs,
            );
        }
    }

    println!("{exprs:#?}");

    let num_splits = exprs
        .values()
        .map(|expr| expr.split_idx(false).unwrap())
        .max()
        .unwrap_or_default()
        + 1;

    let mut module_macro = ModuleMacro {
        cost: num_splits - 1,
        input_map: HashMap::new(),
        output_map: HashMap::new(),
        splits: (0..num_splits)
            .into_iter()
            .map(|idx| Macro {
                name: if idx == 0 {
                    name.to_string()
                } else {
                    format!("_{name}_SPLIT_{idx}")
                },
                inputs: Vec::new(),
                outputs: Vec::new(),
                wrapper: if idx + 1 != num_splits {
                    Some(format!("_{name}_SPLIT_{}", idx + 1))
                } else {
                    None
                },
            })
            .collect(),
    };

    let first_split = module_macro.splits.get_mut(0).unwrap();
    let mut num_inputs = 0;
    for (name, port) in &module.ports {
        if port.direction == PortDirection::Input {
            module_macro.input_map.insert(name.to_string(), num_inputs);
            num_inputs += 1;

            first_split
                .inputs
                .push(port_names.get(name).unwrap().to_string());
        }
    }

    let mut num_outputs = 0;
    for (name, port) in &module.ports {
        if port.direction == PortDirection::Output {
            module_macro
                .output_map
                .insert(name.to_string(), num_outputs);
            num_outputs += 1;

            module_macro.add_expr(
                *port.bits.get(0).unwrap(),
                num_splits - 1,
                &exprs,
                &split_var_exprs,
            );
        }
    }

    module_macro
}

/// Creates a mapping from original port names to macro port names
fn assign_port_names(module: &Module) -> HashMap<String, String> {
    let mut port_map = HashMap::new();

    // Assign non-indexed
    for (port, _) in module.ports.iter().filter(|(name, _)| !name.contains("[")) {
        port_map.insert(port.to_string(), port.to_string());
    }

    // Repeatedly attempt to use different separators until no conflicts with non-indexed ports are found
    let mut separator = String::new();
    loop {
        let mut index_ports_map = HashMap::new();
        for (port, _) in module.ports.iter().filter(|(name, _)| name.contains("[")) {
            let mapped_name = port.replace("]", "").replace("[", &separator);
            index_ports_map.insert(port.to_string(), mapped_name.clone());

            if module.ports.contains_key(&mapped_name) {
                separator = format!("{separator}_");
                continue;
            }
        }

        // No conflicts found
        port_map.extend(index_ports_map);
        break;
    }

    port_map
}

/// Creates expressions for all input connections
fn assign_inputs<'a>(
    module: &'a Module,
    port_names: &HashMap<String, String>,
) -> IndexMap<usize, ExpressionInfo<'a>> {
    let mut known_connections = IndexMap::new();
    for (name, port) in module
        .ports
        .iter()
        .filter(|(_, port)| port.direction == PortDirection::Input)
    {
        for &bit in &port.bits {
            let text = port_names.get(name).unwrap().clone();
            let mut input_vars = HashSet::with_capacity(1);
            input_vars.insert(text.clone());

            known_connections.insert(
                bit,
                ExpressionInfo::Implemented {
                    text,
                    cost: 0,
                    split_idx: 0,
                    split_var: None,
                    input_vars,
                },
            );
        }
    }

    known_connections
}

/// Creates a mapping of output connections to the cell which sets them
fn collect_cell_outputs<'a>(
    module: &'a Module,
    expressions: &mut IndexMap<usize, ExpressionInfo<'a>>,
) {
    for (_name, cell) in &module.cells {
        for &output_connection in cell
            .port_directions
            .iter()
            .filter(|(_, dir)| **dir == PortDirection::Output)
            .flat_map(|(name, _)| cell.connections.get(name).unwrap().iter())
        {
            expressions.insert(
                output_connection,
                ExpressionInfo::Unimplemented { cell: cell },
            );
        }
    }
}

fn count_consumers(module: &Module) -> HashMap<usize, usize> {
    let mut num_consumers = HashMap::new();
    for &connection in module
        .ports
        .iter()
        .filter(|(_, port)| port.direction == PortDirection::Output)
        .flat_map(|(_, port)| port.bits.iter())
    {
        num_consumers
            .entry(connection)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    for connection in module
        .cells
        .iter()
        .flat_map(|(_, cell)| cell.input_connections())
    {
        num_consumers
            .entry(connection)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    num_consumers
}

fn expr_implemented(connection: usize, exprs: &IndexMap<usize, ExpressionInfo>) -> bool {
    let expr = exprs.get(&connection).unwrap();
    matches!(expr, ExpressionInfo::Implemented { .. })
}

fn get_implementable_cell<'a, 'b>(
    connection: usize,
    exprs: &'a IndexMap<usize, ExpressionInfo<'b>>,
) -> Option<usize> {
    let expr = exprs.get(&connection).unwrap();

    // Check if already implemented
    if matches!(expr, ExpressionInfo::Implemented { .. }) {
        return None;
    }

    // Check if implementable
    let cell = expr.cell().unwrap();
    let input_ports = cell.input_connections().collect::<HashSet<_>>();

    if input_ports
        .iter()
        .all(|connection| expr_implemented(*connection, exprs))
    {
        return Some(connection);
    }

    // Check children
    for connection in input_ports {
        if let Some(implementable) = get_implementable_cell(connection, exprs) {
            return Some(implementable);
        }
    }

    unreachable!();
}

fn implement_expr(
    expr_idx: usize,
    exprs: &mut IndexMap<usize, ExpressionInfo>,
    consumer_counts: &HashMap<usize, usize>,
    split_var_exprs: &mut HashMap<String, usize>,
) {
    let cell = exprs.get(&expr_idx).unwrap().cell().unwrap();

    let mut input_texts = HashMap::new();
    let mut input_vars = HashSet::new();
    let mut cost = 1;

    for (port_name, _dir) in cell
        .port_directions
        .iter()
        .filter(|(_, dir)| **dir == PortDirection::Input)
    {
        let connections = cell.connections.get(port_name).unwrap();
        assert_eq!(connections.len(), 1);

        let expr = exprs.get(connections.get(0).unwrap()).unwrap();
        input_texts.insert(port_name.as_str(), expr.text(true).unwrap());

        if let Some(split_var) = expr.split_var() {
            input_vars.insert(split_var.to_string());
        } else {
            cost += expr.cost().unwrap();
            input_vars.extend(expr.input_vars().unwrap().clone());
        }
    }

    let text = match cell.kind.as_str() {
        AND_GATE_NAME => implement_and(input_texts),
        OR_GATE_NAME => implement_or(input_texts),
        XOR_GATE_NAME => implement_xor(input_texts),
        x => unimplemented!("Cell type `{x}`"),
    };

    let consumer_counts = *consumer_counts.get(&expr_idx).unwrap();
    let split_idx = determine_split_idx(cell, &exprs);

    *exprs.get_mut(&expr_idx).unwrap() = ExpressionInfo::Implemented {
        text,
        cost,
        split_idx,
        split_var: if consumer_counts > 1 {
            split_var_exprs.insert(format!("temp_var_{expr_idx}"), expr_idx);
            Some(format!("temp_var_{expr_idx}"))
        } else {
            None
        },
        input_vars,
    };
}

fn determine_split_idx(cell: &Cell, exprs: &IndexMap<usize, ExpressionInfo>) -> usize {
    cell.input_connections()
        .map(|idx| exprs.get(&idx).unwrap().split_idx(true).unwrap())
        .max()
        .unwrap_or(0)
}
