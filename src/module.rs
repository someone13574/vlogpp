use std::collections::{HashMap, VecDeque};

use crate::expr::{Expr, VarID};
use crate::r#macro::{Macro, MacroID};
use crate::netlist::{Cell, Module, Wire};
use crate::registry::Registry;
use crate::scope::MutScope;
use crate::scope::global::GlobalScope;
use crate::{Map, Set};

#[derive(Debug, Clone)]
struct WireInfo {
    pub input_var: Option<VarID>,

    pub expr: Option<Expr>,
    pub downstream_expr: Option<Expr>,
    pub bundled_expr: Option<(VarID, Vec<VarID>)>,

    pub split_delta: Option<usize>,
    pub split_idx_lb: Option<usize>,
    pub split_idx_ub: Option<usize>,

    pub consumers: usize,
    pub input_wires: Set<Wire>,
}

impl WireInfo {
    fn expr_for_lb_split(&self, split_idx_lb: usize) -> Expr {
        if self.split_idx_lb.unwrap() == split_idx_lb {
            self.expr.clone().unwrap()
        } else if self.split_idx_lb.unwrap() + 1 == split_idx_lb && self.bundled_expr.is_some() {
            Expr::Var(self.bundled_expr.as_ref().unwrap().0)
        } else {
            self.downstream_expr
                .as_ref()
                .or(self.expr.as_ref())
                .unwrap()
                .clone()
        }
    }

    fn expr_for_ub_split(&self, split_idx_ub: usize) -> Expr {
        if self.split_idx_ub.unwrap() == split_idx_ub {
            self.expr.clone().unwrap()
        } else if self.split_idx_ub.unwrap() + 1 == split_idx_ub && self.bundled_expr.is_some() {
            Expr::Var(self.bundled_expr.as_ref().unwrap().0)
        } else {
            self.downstream_expr
                .as_ref()
                .or(self.expr.as_ref())
                .unwrap()
                .clone()
        }
    }

    fn downstream_split_idx_lb(&self) -> usize {
        self.split_idx_lb.unwrap() + self.split_delta.unwrap()
    }

    fn placement_split_delta(&self) -> usize {
        // Force bundles to be expanded before the final split
        if self.split_delta.unwrap() == 2 { 2 } else { 0 }
    }
}

#[derive(Clone)]
struct Split {
    pub exprs: Vec<Expr>,
    pub vars: Vec<VarID>,
    pub output_count: usize,
}

pub fn create_module(name: &str, module: &Module, global_scope: &mut GlobalScope) -> MacroID {
    let mut scope = global_scope.new_scope();

    let cell_topo = topo_sort_cells(&module.cells);

    let mut wire_infos = consumer_counts(module);
    let mut var_wires = create_inputs(&mut wire_infos, module, &mut scope);

    for &cell_idx in &cell_topo {
        let (cell_name, cell) = module.cells.iter().nth(cell_idx).unwrap();
        let call_macro = Registry::module(scope.global, &cell.kind)
            .unwrap_or_else(|| panic!("Unknown cell type `{}`", &cell.kind));
        let call_macro_scope = scope.get_macro(call_macro).scope_id;

        let mut input_wires = cell
            .input_connections()
            .map(|(name, wire)| {
                (
                    wire,
                    scope
                        .get_macro(call_macro)
                        .input_position(name, scope.global)
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();

        input_wires.sort_by_key(|(_, idx)| *idx);
        scope
            .get_macro(call_macro)
            .check_inputs(
                input_wires.iter().map(|(_, idx)| *idx).collect(),
                scope.global,
            )
            .unwrap();

        let split_idx_lb = input_wires
            .iter()
            .map(|(wire, _)| wire_infos.get(wire).unwrap().downstream_split_idx_lb())
            .max()
            .unwrap();

        let input_exprs = input_wires
            .iter()
            .map(|(wire, _)| {
                wire_infos
                    .get(wire)
                    .unwrap()
                    .expr_for_lb_split(split_idx_lb)
                    .clone()
            })
            .collect::<Vec<_>>();

        let expr = Expr::Call {
            r#macro: Box::new(Expr::Macro(call_macro)),
            args: input_exprs,
        };

        let mut output_wires = cell
            .output_connections()
            .map(|(name, wire)| {
                (
                    wire,
                    scope
                        .get_scope(call_macro_scope)
                        .local()
                        .output_names
                        .as_ref()
                        .unwrap()
                        .iter()
                        .position(|output_name| output_name == name)
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        output_wires.sort_by_key(|(_, idx)| *idx);

        let total_consumers: usize = output_wires
            .iter()
            .map(|(wire, _)| wire_infos.get(wire).unwrap().consumers)
            .sum();

        if output_wires.len() > 1 {
            let bundle_var = scope.new_var(cell_name, false, false, None);
            let expanded_vars = (0..output_wires.len())
                .map(|_| scope.new_var("bt", false, false, Some(bundle_var)))
                .collect::<Vec<_>>();
            let bundle_expr = Some((bundle_var, expanded_vars.clone()));

            for ((wire, _), &temp) in output_wires.iter().zip(expanded_vars.iter()) {
                let wire_info = wire_infos.get_mut(wire).unwrap();
                wire_info.expr = Some(expr.clone());
                wire_info.split_idx_lb = Some(split_idx_lb);
                wire_info.split_delta = Some(2);
                wire_info
                    .input_wires
                    .extend(input_wires.iter().map(|(wire, _)| wire));
                wire_info.downstream_expr = Some(Expr::Var(temp));
                wire_info.bundled_expr = bundle_expr.clone();

                var_wires.insert(temp, *wire);
            }
        } else {
            let (wire, _) = output_wires.first().unwrap();
            let wire_info = wire_infos.get_mut(wire).unwrap();
            wire_info.expr = Some(expr.clone());
            wire_info.split_idx_lb = Some(split_idx_lb);
            wire_info
                .input_wires
                .extend(input_wires.iter().map(|(wire, _)| wire));

            if total_consumers > 1 {
                let var_id = scope.new_var("t", false, false, None);
                var_wires.insert(var_id, *wire);
                wire_info.downstream_expr = Some(Expr::Var(var_id));
                wire_info.split_delta = Some(1);
            } else {
                wire_info.split_delta = Some(0);
            }
        }
    }

    let max_split = wire_infos
        .values()
        .map(|info| info.split_idx_lb.unwrap() + info.placement_split_delta())
        .max()
        .unwrap_or_default();
    compute_split_upper_bounds(max_split, &mut wire_infos);

    let mut splits = vec![
        Split {
            exprs: Vec::new(),
            vars: Vec::new(),
            output_count: 0
        };
        max_split + 1
    ];

    splits[0].vars = module
        .input_ports()
        .map(|(_, port)| wire_infos.get(&port.wire).unwrap().input_var.unwrap())
        .collect();

    scope.local().output_names = Some(Vec::new());
    for (name, port) in module.output_ports() {
        add_to_split(port.wire, max_split, &wire_infos, &var_wires, &mut splits);
        scope
            .local()
            .output_names
            .as_mut()
            .unwrap()
            .push(name.to_string());
    }

    let ids = splits
        .into_iter()
        .enumerate()
        .map(|(idx, split)| {
            scope.new_macro(Macro {
                scope_id: scope.id,
                name: scope.get_alias(name, false),
                expr: Expr::List(split.exprs, ", "),
                inputs: split.vars,
                variadicified_vars: None,
                calling_split: None,
                doc_name: if idx == 0 {
                    Some(name.to_string())
                } else {
                    None
                },
            })
        })
        .collect::<Vec<_>>();

    let mut next_split = None;
    for &macro_id in ids.iter().rev() {
        if let Some(next_split) = next_split {
            let r#macro = scope.get_mut_macro(macro_id);
            let Expr::List(inner, _sep) = r#macro.expr.clone() else {
                unreachable!();
            };

            r#macro.expr = Expr::Call {
                r#macro: Box::new(Expr::Macro(next_split)),
                args: inner,
            };

            scope.get_mut_macro(next_split).calling_split = Some(macro_id);
        }

        next_split = Some(macro_id);
    }

    for &macro_id in ids.iter().rev() {
        Macro::sort_passthrough_vars(macro_id, &mut scope);
    }

    *ids.first().unwrap()
}

fn topo_sort_cells(cells: &Map<String, Cell>) -> Vec<usize> {
    let mut children = vec![Vec::new(); cells.len()];
    let mut incoming = vec![0_usize; cells.len()];

    let connection_cells = cells
        .values()
        .enumerate()
        .flat_map(|(cell_idx, cell)| {
            cell.output_connections()
                .map(move |(_, wire)| (wire, cell_idx))
        })
        .collect::<Map<Wire, usize>>();

    for (consumer_idx, cell) in cells.values().enumerate() {
        for (_, producer_wire) in cell.input_connections() {
            if let Some(&producer_idx) = connection_cells.get(&producer_wire) {
                children[producer_idx].push(consumer_idx);
                incoming[consumer_idx] += 1;
            }
        }
    }

    let mut queue = VecDeque::new();
    for (cell_idx, &degree) in incoming.iter().enumerate() {
        // Get outputs
        if degree == 0 {
            queue.push_back(cell_idx);
        }
    }

    let mut topo = Vec::new();
    while let Some(cell_idx) = queue.pop_front() {
        topo.push(cell_idx);
        for &child_idx in &children[cell_idx] {
            let degree = &mut incoming[child_idx];
            *degree -= 1;

            if *degree == 0 {
                queue.push_back(child_idx);
            }
        }
    }

    assert_eq!(topo.len(), cells.len(), "Netlist topology is not a DAG");
    topo
}

fn consumer_counts(module: &Module) -> Map<Wire, WireInfo> {
    let mut consumer_counts = Map::new();
    for producer in module.output_ports().map(|(_name, port)| port.wire).chain(
        module
            .cells
            .values()
            .flat_map(|cell| cell.input_connections().map(|(_, wire)| wire)),
    ) {
        consumer_counts
            .entry(producer)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    consumer_counts
        .into_iter()
        .map(|(wire, count)| {
            {
                let mut wire_info = WireInfo {
                    input_var: None,
                    expr: None,
                    downstream_expr: None,
                    bundled_expr: None,
                    split_delta: None,
                    split_idx_lb: None,
                    split_idx_ub: None,
                    consumers: count,
                    input_wires: Set::new(),
                };

                match wire {
                    Wire::Const(constant) => {
                        wire_info.expr = Some(Expr::Text(if constant {
                            "1".to_string()
                        } else {
                            "0".to_string()
                        }));
                        wire_info.split_idx_lb = Some(0);
                        wire_info.split_idx_ub = Some(0); // TODO: Not this
                        wire_info.split_delta = Some(0);
                    }
                    Wire::Wire(_) => {}
                }

                (wire, wire_info)
            }
        })
        .collect()
}

fn create_inputs(
    wire_infos: &mut Map<Wire, WireInfo>,
    module: &Module,
    scope: &mut MutScope,
) -> Map<VarID, Wire> {
    let mut var_wires = Map::new();

    for (name, port) in module.input_ports() {
        let wire_info = wire_infos.get_mut(&port.wire).unwrap();
        wire_info.split_idx_lb = Some(0);
        wire_info.split_delta = Some(0);

        let var_id = scope.new_var(name, true, false, None);
        wire_info.input_var = Some(var_id);
        wire_info.expr = Some(Expr::Var(var_id));
        var_wires.insert(var_id, port.wire);
    }

    var_wires
}

fn compute_split_upper_bounds(max_split: usize, wire_infos: &mut Map<Wire, WireInfo>) {
    if max_split < 2 || wire_infos.is_empty() {
        wire_infos.values_mut().for_each(|info| {
            info.split_idx_ub = info.split_idx_lb;
        });

        return;
    }

    // Build adjacency
    let mut children = Map::new();
    let mut incoming = Map::new();
    for &wire in wire_infos.keys() {
        children.insert(wire, Set::new());
        incoming.insert(wire, 0);
    }

    for (consumer_wire, consumer_info) in wire_infos.iter() {
        for &producer_wire in &consumer_info.input_wires {
            children
                .get_mut(&producer_wire)
                .unwrap()
                .insert(*consumer_wire);
            *incoming.get_mut(consumer_wire).unwrap() += 1;
        }
    }

    // Topo sort
    let mut queue = VecDeque::new();
    for (&wire, &degree) in &incoming {
        if degree == 0 {
            queue.push_back(wire);
        }
    }

    let mut topo = Vec::new();
    while let Some(wire) = queue.pop_front() {
        topo.push(wire);
        if let Some(wire_children) = children.get(&wire) {
            for child in wire_children {
                let degree = incoming.get_mut(child).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(*child);
                }
            }
        }
    }

    assert_eq!(topo.len(), wire_infos.len());

    // Bundle to wires map
    let mut bundle_groups = HashMap::new();
    for (&wire, wire_info) in wire_infos.iter() {
        if let Some((bundle_var, _)) = &wire_info.bundled_expr {
            bundle_groups
                .entry(*bundle_var)
                .and_modify(|wires: &mut Vec<Wire>| wires.push(wire))
                .or_insert(vec![wire]);
        }
    }

    // Compute upper bounds
    for wire_info in wire_infos.values_mut() {
        if wire_info.input_var.is_some() {
            wire_info.split_idx_ub = Some(0);
        } else {
            wire_info.split_idx_ub = Some(max_split - wire_info.placement_split_delta());
        }
    }

    for wire in topo.iter().rev() {
        let delta = wire_infos.get(wire).unwrap().split_delta.unwrap();
        if let Some(wire_children) = children.get(wire) {
            for child in wire_children {
                let ub = wire_infos
                    .get(child)
                    .unwrap()
                    .split_idx_ub
                    .unwrap()
                    .saturating_sub(delta);

                if ub < wire_infos.get(wire).unwrap().split_idx_ub.unwrap() {
                    if let Some((bundle_var, _)) = wire_infos
                        .get(wire)
                        .and_then(|info| info.bundled_expr.as_ref())
                    {
                        for bundle_wire in bundle_groups.get(bundle_var).unwrap() {
                            assert!(
                                wire_infos
                                    .get(bundle_wire)
                                    .unwrap()
                                    .split_idx_ub
                                    .is_none_or(|old_ub| ub < old_ub)
                            );
                            wire_infos.get_mut(bundle_wire).unwrap().split_idx_ub = Some(ub);
                        }
                    } else {
                        wire_infos.get_mut(wire).unwrap().split_idx_ub = Some(ub);
                    }
                }
            }
        }
    }
}

fn add_to_split(
    wire: Wire,
    target_split_idx: usize,
    wire_infos: &Map<Wire, WireInfo>,
    var_wires: &Map<VarID, Wire>,
    splits: &mut Vec<Split>,
) {
    let info = wire_infos.get(&wire).unwrap();
    let expr = info.expr_for_ub_split(target_split_idx);

    splits[target_split_idx].exprs.push(expr.clone());
    splits[target_split_idx].output_count += 1;

    for var_id in expr.vars() {
        let var_wire = var_wires.get(&var_id).unwrap();
        let var_info = wire_infos.get(var_wire).unwrap();
        let mut prev_input_idx = None;

        for (split_idx, split) in splits
            .iter_mut()
            .enumerate()
            .take(target_split_idx + 1)
            .rev()
        {
            // Add source expr
            if var_info.split_idx_ub.unwrap() == split_idx && var_info.split_delta.unwrap() != 0 {
                if prev_input_idx.is_some_and(|prev_input_idx| prev_input_idx == split.output_count)
                {
                    add_to_split(*var_wire, split_idx, wire_infos, var_wires, splits);
                }
                break;
            }

            // Propagation
            if var_info.split_idx_ub.unwrap() + 1 == split_idx && var_info.bundled_expr.is_some() {
                // Add bundle var to expr
                let (bundle_var, expanded_vars) = var_info.bundled_expr.as_ref().unwrap();
                if prev_input_idx.is_some_and(|prev_input_idx| {
                    prev_input_idx + 1 == split.output_count + expanded_vars.len()
                }) {
                    split.exprs.push(Expr::Var(*bundle_var));
                    split.output_count += expanded_vars.len();
                }

                // Add bundle var to inputs
                if split.vars.contains(bundle_var) {
                    break;
                } else {
                    prev_input_idx = Some(split.vars.len());
                    split.vars.push(*bundle_var);
                }
            } else if let Some((_bundle_var, expanded_vars)) = var_info.bundled_expr.as_ref() {
                // Add expanded vars to expr
                if prev_input_idx.is_some_and(|prev_input_idx| {
                    prev_input_idx + 1 == split.output_count + expanded_vars.len()
                }) {
                    split
                        .exprs
                        .extend(expanded_vars.iter().map(|var| Expr::Var(*var)));
                    split.output_count += expanded_vars.len();
                }

                // Add expanded vars to input
                if expanded_vars
                    .iter()
                    .any(|expanded| split.vars.contains(expanded))
                {
                    break;
                } else {
                    prev_input_idx = Some(split.vars.len() + expanded_vars.len() - 1);
                    split.vars.extend(expanded_vars);
                }
            } else {
                // Add single var to expr
                if prev_input_idx.is_some_and(|prev_input_idx| prev_input_idx == split.output_count)
                {
                    split
                        .exprs
                        .push(var_info.expr_for_ub_split(split_idx).clone());
                    split.output_count += 1;
                }

                if split.vars.contains(&var_id) {
                    break;
                } else {
                    // Add single var to inputs
                    prev_input_idx = Some(split.vars.len());
                    split.vars.push(var_id);
                }
            }
        }
    }
}
