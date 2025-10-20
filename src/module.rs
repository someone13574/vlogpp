use std::collections::VecDeque;

use crate::expr::{Expr, VarID};
use crate::r#macro::{Macro, MacroID};
use crate::netlist::{Cell, Module, Wire};
use crate::registry::Registry;
use crate::scope::global::GlobalScope;
use crate::scope::{MutScope, Scope};
use crate::{Map, Set};

#[derive(Debug, Clone)]
struct WireInfo {
    pub input_var: Option<VarID>,

    pub expr: Option<Expr>,
    pub downstream_expr: Option<Expr>,
    pub bundled_expr: Option<(VarID, Expr, usize)>,

    pub split_delta: Option<usize>,
    pub split_idx_lb: Option<usize>,
    pub split_idx_ub: Option<usize>,

    pub consumers: usize,
    pub input_wires: Set<Wire>,
}

impl WireInfo {
    fn expr_for_lb_split(&self, split_idx_lb: usize) -> &Expr {
        if self.split_idx_lb.unwrap() == split_idx_lb {
            self.expr.as_ref().unwrap()
        } else if self.split_idx_lb.unwrap() + 1 == split_idx_lb && self.bundled_expr.is_some() {
            &self.bundled_expr.as_ref().unwrap().1
        } else {
            self.downstream_expr
                .as_ref()
                .or(self.expr.as_ref())
                .unwrap()
        }
    }

    fn expr_for_ub_split(&self, split_idx_ub: usize) -> &Expr {
        if self.split_idx_ub.unwrap() == split_idx_ub {
            self.expr.as_ref().unwrap()
        } else if self.split_idx_ub.unwrap() + 1 == split_idx_ub && self.bundled_expr.is_some() {
            &self.bundled_expr.as_ref().unwrap().1
        } else {
            self.downstream_expr
                .as_ref()
                .or(self.expr.as_ref())
                .unwrap()
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

    scope.set_outputs(
        module
            .output_ports()
            .map(|(name, _)| name.to_string())
            .collect(),
    );

    for &cell_idx in &cell_topo {
        let (_cell_name, cell) = module.cells.iter().nth(cell_idx).unwrap();
        let call_macro = Registry::module(scope.global, &cell.kind)
            .unwrap_or_else(|| panic!("Unknown cell type `{}`", &cell.kind));

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
        add_reg_macro_prev_output(&mut input_wires, cell, call_macro, scope.scope(), module);

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

        let output_wires = cell
            .output_connections()
            .map(|(_, wire)| wire)
            .collect::<Vec<_>>();
        let total_consumers: usize = output_wires
            .iter()
            .map(|wire| wire_infos.get(wire).unwrap().consumers)
            .sum();

        let bundle_var = if output_wires.len() > 1 {
            Some(scope.new_var(&format!("bundleof{}_", output_wires.len()), false, false))
        } else {
            None
        };

        for &wire in &output_wires {
            let wire_info = wire_infos.get_mut(&wire).unwrap();
            wire_info.expr = Some(expr.clone());
            wire_info.split_idx_lb = Some(split_idx_lb);
            wire_info
                .input_wires
                .extend(input_wires.iter().map(|(wire, _)| wire));

            if total_consumers > 1 || cell.output_connections().count() > 1 {
                let var_id = scope.new_var("t", false, false);
                var_wires.insert(var_id, wire);
                wire_info.downstream_expr = Some(Expr::Var(var_id));

                if let Some(bundle_var) = bundle_var {
                    wire_info.bundled_expr =
                        Some((bundle_var, Expr::Var(bundle_var), output_wires.len()));
                    wire_info.split_delta = Some(2);
                } else {
                    wire_info.split_delta = Some(1);
                }
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

    for (_name, port) in module.output_ports() {
        add_to_split(port.wire, max_split, &wire_infos, &var_wires, &mut splits);
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
                output_to_input: None,
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
        }

        next_split = Some(macro_id);
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
            (
                wire,
                WireInfo {
                    input_var: None,
                    expr: None,
                    downstream_expr: None,
                    bundled_expr: None,
                    split_delta: None,
                    split_idx_lb: None,
                    split_idx_ub: None,
                    consumers: count,
                    input_wires: Set::new(),
                },
            )
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

        let var_id = scope.new_var(name, true, false);
        wire_info.input_var = Some(var_id);
        wire_info.expr = Some(Expr::Var(var_id));
        var_wires.insert(var_id, port.wire);
    }

    var_wires
}

fn add_reg_macro_prev_output(
    inputs: &mut Vec<(Wire, usize)>,
    cell: &Cell,
    macro_id: MacroID,
    scope: Scope,
    module: &Module,
) {
    if let Some(map_output_to_input_idx) = scope.get_macro(macro_id).output_to_input {
        assert_eq!(cell.output_connections().count(), 1);

        let (_, output_wire) = cell.output_connections().next().unwrap();
        let (output_name, _) = module
            .output_ports()
            .find(|(_, port)| output_wire == port.wire)
            .unwrap();
        let input_name = format!("{output_name}.i");
        let input_wire = module.ports.get(&input_name).unwrap().wire;

        inputs.push((input_wire, map_output_to_input_idx));
    }
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
                    wire_infos.get_mut(wire).unwrap().split_idx_ub = Some(ub);
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
            if var_info.split_idx_ub.unwrap() == split_idx && var_info.split_delta.unwrap() != 0 {
                if prev_input_idx.is_some_and(|prev_input_idx| prev_input_idx == split.output_count)
                {
                    add_to_split(*var_wire, split_idx, wire_infos, var_wires, splits);
                }
                break;
            }

            if var_info.split_idx_ub.unwrap() + 1 == split_idx && var_info.bundled_expr.is_some() {
                let (bundle_var, bundle_var_expr, expanded_len) =
                    var_info.bundled_expr.as_ref().unwrap();
                if prev_input_idx.is_some_and(|prev_input_idx| {
                    prev_input_idx + 1 == split.output_count + expanded_len
                }) {
                    split.exprs.push(bundle_var_expr.clone());
                    split.output_count += expanded_len;
                }

                if let Some(idx) = split.vars.iter().position(|input| input == bundle_var) {
                    prev_input_idx = Some(idx);
                } else {
                    prev_input_idx = Some(split.vars.len());
                    split.vars.push(*bundle_var);
                }
            } else {
                if prev_input_idx.is_some_and(|prev_input_idx| prev_input_idx == split.output_count)
                {
                    split
                        .exprs
                        .push(var_info.expr_for_ub_split(split_idx).clone());
                    split.output_count += 1;
                }

                if let Some(idx) = split.vars.iter().position(|input| *input == var_id) {
                    prev_input_idx = Some(idx);
                } else {
                    prev_input_idx = Some(split.vars.len());
                    split.vars.push(var_id);
                }
            }
        }
    }
}
