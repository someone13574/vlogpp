use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::{self, Display},
};

use ordermap::OrderSet;

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
        exprs: &HashMap<usize, ImplementedExpr>,
        split_var_exprs: &HashMap<String, usize>,
    ) {
        let expr = exprs.get(&expr_idx).unwrap();
        self.splits
            .get_mut(add_split_idx)
            .unwrap()
            .outputs
            .push(expr.text.clone());
        self.cost += expr.cost;

        let skip = self.splits.len() - add_split_idx - 1;
        for var in &expr.input_vars {
            let split_var_expr = split_var_exprs
                .get(var)
                .and_then(|expr_idx| exprs.get(expr_idx));
            let mut prev_input_idx = None;

            for (split_idx, split) in self.splits.iter_mut().enumerate().rev().skip(skip) {
                if let Some(split_var_expr) = split_var_expr {
                    if split_var_expr.split_idx == split_idx {
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
struct ImplementedExpr {
    text: String,
    cost: usize,
    split_idx: usize,
    split_var: Option<String>,
    input_vars: OrderSet<String>,
}

impl ImplementedExpr {
    fn downstream_split_idx(&self) -> usize {
        if self.split_var.is_some() {
            self.split_idx + 1
        } else {
            self.split_idx
        }
    }
}

#[derive(Debug)]
struct UnimplementedExpr<'a> {
    cell: &'a Cell,
}

pub fn build_module_macro(name: &str, module: &Module) -> ModuleMacro {
    let port_names = assign_port_names(module);
    let mut split_var_exprs = HashMap::new();
    let mut imp_exprs = assign_inputs(module, &port_names);
    let mut unimp_exprs = collect_cell_outputs(module);

    let consumer_counts = count_consumers(module);

    let expr_idxs = unimp_exprs.keys().copied().collect::<Vec<usize>>();
    for expr_idx in expr_idxs {
        while let Some(implementable_expr_idx) =
            get_implementable_cell(expr_idx, &imp_exprs, &unimp_exprs)
        {
            implement_expr(
                implementable_expr_idx,
                &mut imp_exprs,
                &mut unimp_exprs,
                &consumer_counts,
                &mut split_var_exprs,
            );
        }
    }

    assert!(unimp_exprs.is_empty());
    println!("{imp_exprs:#?}");

    let num_splits = imp_exprs
        .values()
        .map(|expr| expr.split_idx)
        .max()
        .unwrap_or_default()
        + 1;

    defer_split_indices(num_splits, &mut imp_exprs, &mut split_var_exprs);

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
                &imp_exprs,
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
fn assign_inputs(
    module: &Module,
    port_names: &HashMap<String, String>,
) -> HashMap<usize, ImplementedExpr> {
    let mut known_connections = HashMap::new();
    for (name, port) in module
        .ports
        .iter()
        .filter(|(_, port)| port.direction == PortDirection::Input)
    {
        for &bit in &port.bits {
            let text = port_names.get(name).unwrap().clone();
            let mut input_vars = OrderSet::with_capacity(1);
            input_vars.insert(text.clone());

            known_connections.insert(
                bit,
                ImplementedExpr {
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
fn collect_cell_outputs<'a>(module: &'a Module) -> HashMap<usize, UnimplementedExpr<'a>> {
    let mut unimp_exprs = HashMap::new();

    for (_name, cell) in &module.cells {
        for &output_connection in cell
            .port_directions
            .iter()
            .filter(|(_, dir)| **dir == PortDirection::Output)
            .flat_map(|(name, _)| cell.connections.get(name).unwrap().iter())
        {
            unimp_exprs.insert(output_connection, UnimplementedExpr { cell: cell });
        }
    }

    unimp_exprs
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

fn get_implementable_cell<'a, 'b>(
    connection: usize,
    imp_exprs: &'a HashMap<usize, ImplementedExpr>,
    unimp_exprs: &'a HashMap<usize, UnimplementedExpr<'b>>,
) -> Option<usize> {
    let Some(unimp_expr) = unimp_exprs.get(&connection) else {
        return None;
    };

    // Check if implementable
    let input_ports = unimp_expr.cell.input_connections().collect::<HashSet<_>>();

    if input_ports
        .iter()
        .all(|connection| imp_exprs.contains_key(connection))
    {
        return Some(connection);
    }

    // Check children
    for connection in input_ports {
        if let Some(implementable) = get_implementable_cell(connection, imp_exprs, unimp_exprs) {
            return Some(implementable);
        }
    }

    unreachable!();
}

fn implement_expr(
    unimp_expr_idx: usize,
    imp_exprs: &mut HashMap<usize, ImplementedExpr>,
    unimp_exprs: &mut HashMap<usize, UnimplementedExpr>,
    consumer_counts: &HashMap<usize, usize>,
    split_var_exprs: &mut HashMap<String, usize>,
) {
    let cell = unimp_exprs.get(&unimp_expr_idx).unwrap().cell;

    let mut input_texts = HashMap::new();
    let mut input_vars = OrderSet::new();
    let mut cost = 1;

    for (port_name, _dir) in cell
        .port_directions
        .iter()
        .filter(|(_, dir)| **dir == PortDirection::Input)
    {
        let connections = cell.connections.get(port_name).unwrap();
        assert_eq!(connections.len(), 1);

        let input_expr = imp_exprs.get(connections.get(0).unwrap()).unwrap();
        if let Some(split_var) = &input_expr.split_var {
            input_texts.insert(port_name.as_str(), split_var.as_str());
            input_vars.insert(split_var.to_string());
        } else {
            input_texts.insert(port_name.as_str(), input_expr.text.as_str());
            input_vars.extend(input_expr.input_vars.clone());
            cost += input_expr.cost;
        }
    }

    let text = match cell.kind.as_str() {
        AND_GATE_NAME => implement_and(input_texts),
        OR_GATE_NAME => implement_or(input_texts),
        XOR_GATE_NAME => implement_xor(input_texts),
        x => unimplemented!("Cell type `{x}`"),
    };

    let consumer_counts = *consumer_counts.get(&unimp_expr_idx).unwrap();
    let split_idx = determine_split_idx(cell, &imp_exprs);

    imp_exprs.insert(
        unimp_expr_idx,
        ImplementedExpr {
            text,
            cost,
            split_idx,
            split_var: if consumer_counts > 1 {
                split_var_exprs.insert(format!("tmp{unimp_expr_idx}"), unimp_expr_idx);
                Some(format!("tmp{unimp_expr_idx}"))
            } else {
                None
            },
            input_vars,
        },
    );

    unimp_exprs.remove(&unimp_expr_idx);
}

fn determine_split_idx(cell: &Cell, exprs: &HashMap<usize, ImplementedExpr>) -> usize {
    cell.input_connections()
        .map(|idx| exprs.get(&idx).unwrap().downstream_split_idx())
        .max()
        .unwrap_or(0)
}

fn defer_split_indices(
    num_splits: usize,
    exprs: &mut HashMap<usize, ImplementedExpr>,
    split_var_exprs: &mut HashMap<String, usize>,
) {
    if num_splits < 3 || exprs.is_empty() {
        return;
    }

    // Build adjacency
    let mut children = HashMap::new();
    let mut incoming = HashMap::new();
    for expr_idx in exprs.keys() {
        children.insert(expr_idx, Vec::new());
        incoming.insert(expr_idx, 0);
    }

    for (consumer_idx, ImplementedExpr { input_vars, .. }) in exprs.iter() {
        for var in input_vars {
            if let Some(producer_idx) = split_var_exprs.get(var) {
                children.get_mut(producer_idx).unwrap().push(*consumer_idx);
                *incoming.get_mut(consumer_idx).unwrap() += 1;
            }
        }
    }

    // Topo sort
    let mut queue = VecDeque::new();
    for (&&expr_idx, &degree) in &incoming {
        // Get outputs
        if degree == 0 {
            queue.push_back(expr_idx);
        }
    }

    let mut topo = Vec::new();
    while let Some(expr_idx) = queue.pop_front() {
        topo.push(expr_idx);
        if let Some(expr_children) = children.get(&expr_idx) {
            for child in expr_children {
                let degree = incoming.get_mut(child).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(*child);
                }
            }
        }
    }

    assert_eq!(topo.len(), exprs.len());

    // Compute upper bound splits
    let mut upper_bounds = HashMap::new();
    for expr_idx in &topo {
        upper_bounds.insert(
            *expr_idx,
            if exprs.get(expr_idx).unwrap().cost == 0 {
                // Inputs must be in split 0
                0
            } else {
                num_splits - 1
            },
        );
    }

    for expr_idx in topo.iter().rev() {
        if let Some(expr_children) = children.get(&expr_idx) {
            for child in expr_children {
                let delta = if exprs.get(expr_idx).unwrap().split_var.is_some() {
                    // Split must decrease if a temp var
                    1
                } else {
                    0
                };

                let child_upper_bound = upper_bounds.get(child).copied().unwrap_or(num_splits - 1);
                let candidate = child_upper_bound.saturating_sub(delta);
                if candidate
                    < upper_bounds
                        .get(expr_idx)
                        .copied()
                        .unwrap_or(num_splits - 1)
                {
                    upper_bounds.insert(*expr_idx, candidate);
                }
            }
        }
    }

    for (idx, expr) in exprs.iter_mut() {
        expr.split_idx = upper_bounds.get(idx).copied().unwrap_or(num_splits - 1);
    }
}
