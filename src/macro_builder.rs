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

#[derive(Debug)]
pub struct Macro {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

impl Display for Macro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "#define {}({}) {}",
            self.name,
            self.inputs.join(", "),
            self.outputs.join(", ")
        )
    }
}

#[derive(Debug)]
pub struct ModuleMacro {
    pub cost: usize,
    pub input_map: HashMap<String, usize>,
    pub output_map: HashMap<String, usize>,
    pub module_macro: Macro,
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
        writeln!(f, "{}", self.module_macro)
    }
}

#[derive(Debug)]
enum ExpressionInfo<'a> {
    Implemented { text: String, cost: usize },
    Unimplemented { cell: &'a Cell },
}

impl<'a> ExpressionInfo<'a> {
    fn text(&self) -> Option<&str> {
        match self {
            Self::Implemented { text, cost: _ } => Some(text),
            _ => None,
        }
    }

    fn cost(&self) -> Option<usize> {
        match self {
            Self::Implemented { text: _, cost } => Some(*cost),
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
    let mut exprs = assign_inputs(module, &port_names);
    collect_cell_outputs(module, &mut exprs);

    for idx in 0..exprs.len() {
        let (&expr_idx, _) = exprs.get_index(idx).unwrap();
        while let Some(implementable_expr_idx) = get_implementable_cell(expr_idx, &exprs) {
            implement_expr(implementable_expr_idx, &mut exprs);
        }
    }

    let mut module_macro = ModuleMacro {
        cost: 0,
        input_map: HashMap::new(),
        output_map: HashMap::new(),
        module_macro: Macro {
            name: name.to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        },
    };

    for (name, port) in &module.ports {
        if port.direction == PortDirection::Input {
            module_macro
                .input_map
                .insert(name.to_string(), module_macro.module_macro.inputs.len());
            module_macro
                .module_macro
                .inputs
                .push(port_names.get(name).unwrap().to_string());
        } else {
            let expr = exprs.get(port.bits.get(0).unwrap()).unwrap();

            module_macro
                .output_map
                .insert(name.to_string(), module_macro.module_macro.outputs.len());
            module_macro
                .module_macro
                .outputs
                .push(expr.text().unwrap().to_string());
            module_macro.cost += expr.cost().unwrap();
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
            known_connections.insert(
                bit,
                ExpressionInfo::Implemented {
                    text: port_names.get(name).unwrap().clone(),
                    cost: 0,
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
    let input_ports = cell
        .port_directions
        .iter()
        .filter(|(_, dir)| **dir == PortDirection::Input)
        .flat_map(|(name, _)| cell.connections.get(name).unwrap().iter())
        .collect::<HashSet<_>>();

    if input_ports
        .iter()
        .all(|connection| expr_implemented(**connection, exprs))
    {
        return Some(connection);
    }

    // Check children
    for &connection in input_ports {
        if let Some(implementable) = get_implementable_cell(connection, exprs) {
            return Some(implementable);
        }
    }

    unreachable!();
}

fn implement_expr(expr_idx: usize, exprs: &mut IndexMap<usize, ExpressionInfo>) {
    let cell = exprs.get(&expr_idx).unwrap().cell().unwrap();
    let inputs = cell
        .port_directions
        .iter()
        .filter(|(_, dir)| **dir == PortDirection::Input)
        .map(|(name, _)| {
            let id = cell.connections.get(name).unwrap().get(0).unwrap();
            (name.as_str(), exprs.get(id).unwrap().text().unwrap())
        })
        .collect::<HashMap<&str, &str>>();

    let text = match cell.kind.as_str() {
        AND_GATE_NAME => implement_and(inputs),
        OR_GATE_NAME => implement_or(inputs),
        XOR_GATE_NAME => implement_xor(inputs),
        x => unimplemented!("Cell type `{x}`"),
    };

    let cost = cell
        .port_directions
        .iter()
        .filter(|(_, dir)| **dir == PortDirection::Input)
        .map(|(name, _)| {
            exprs
                .get(cell.connections.get(name).unwrap().get(0).unwrap())
                .unwrap()
                .cost()
                .unwrap()
        })
        .sum::<usize>()
        + 1;
    *exprs.get_mut(&expr_idx).unwrap() = ExpressionInfo::Implemented { text, cost };
}
