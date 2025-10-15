use std::collections::VecDeque;

use ordermap::{OrderMap, OrderSet};

use crate::expr::{ExprContent, ExprID, VarID};
use crate::global_scope::GlobalScope;
use crate::local_scope::LocalScope;
use crate::r#macro::MacroID;
use crate::yosys::{Cell, Module, Wire};

#[derive(Debug, Clone)]
struct WireInfo {
    pub input_var: Option<VarID>,

    pub expr: Option<ExprID>,
    pub downstream_expr: Option<ExprID>,
    pub bundled_expr: Option<(VarID, ExprID, usize)>,

    pub split_delta: Option<usize>,
    pub split_idx_lb: Option<usize>,
    pub split_idx_ub: Option<usize>,

    pub consumers: usize,
    pub input_wires: OrderSet<Wire>,
}

impl WireInfo {
    fn expr_for_lb_split(&self, split_idx_lb: usize) -> ExprID {
        if self.split_idx_lb.unwrap() == split_idx_lb {
            self.expr.unwrap()
        } else if self.split_idx_lb.unwrap() + 1 == split_idx_lb && self.bundled_expr.is_some() {
            self.bundled_expr.unwrap().1
        } else {
            self.downstream_expr.or(self.expr).unwrap()
        }
    }

    fn expr_for_ub_split(&self, split_idx_ub: usize) -> ExprID {
        if self.split_idx_ub.unwrap() == split_idx_ub {
            self.expr.unwrap()
        } else if self.split_idx_ub.unwrap() + 1 == split_idx_ub && self.bundled_expr.is_some() {
            self.bundled_expr.unwrap().1
        } else {
            self.downstream_expr.or(self.expr).unwrap()
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
    pub exprs: Vec<ExprID>,
    pub vars: Vec<VarID>,
    pub output_count: usize,
}

pub fn create_module(name: &str, module: &Module, global_scope: &mut GlobalScope) -> MacroID {
    let scope_id = global_scope.new_local_scope();

    let cell_topo = topo_sort_cells(&module.cells);

    let mut wire_infos = consumer_counts(module);
    let mut var_wires = create_inputs(
        &mut wire_infos,
        module,
        global_scope.get_mut_scope(scope_id),
    );

    global_scope.get_mut_scope(scope_id).output_names = Some(
        module
            .output_ports()
            .map(|(name, _)| name.to_string())
            .collect(),
    );

    for &cell_idx in &cell_topo {
        let (_cell_name, cell) = module.cells.get_index(cell_idx).unwrap();
        let call_module = global_scope
            .get_module(&cell.kind)
            .expect(&format!("Unknown cell type `{}`", &cell.kind));

        let mut input_wires = cell
            .input_connections()
            .map(|(name, wire)| {
                (
                    wire,
                    global_scope
                        .macros
                        .get(&call_module)
                        .unwrap()
                        .input_position(name, global_scope)
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        add_reg_macro_prev_output(&mut input_wires, cell, call_module, global_scope, module);

        input_wires.sort_by_key(|(_, idx)| *idx);
        global_scope
            .macros
            .get(&call_module)
            .unwrap()
            .check_inputs(
                input_wires.iter().map(|(_, idx)| *idx).collect(),
                global_scope,
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
            })
            .collect::<Vec<_>>();

        let content = ExprContent::List(input_exprs);
        let expr_id = global_scope
            .get_mut_scope(scope_id)
            .new_expr(content, Some(call_module));

        let output_wires = cell
            .output_connections()
            .map(|(_, wire)| wire)
            .collect::<Vec<_>>();
        let total_consumers: usize = output_wires
            .iter()
            .map(|wire| wire_infos.get(wire).unwrap().consumers)
            .sum();

        let bundle_var_expr = if output_wires.len() > 1 {
            let var_id = global_scope
                .get_mut_scope(scope_id)
                .new_var(&format!("bundleof{}_", output_wires.len()), false);
            let expr_id = global_scope
                .get_mut_scope(scope_id)
                .new_expr(ExprContent::Var(var_id), None);
            Some((var_id, expr_id))
        } else {
            None
        };

        for &wire in &output_wires {
            let wire_info = wire_infos.get_mut(&wire).unwrap();
            wire_info.expr = Some(expr_id);
            wire_info.split_idx_lb = Some(split_idx_lb);
            wire_info
                .input_wires
                .extend(input_wires.iter().map(|(wire, _)| wire));

            if total_consumers > 1 || cell.output_connections().count() > 1 {
                let var_id = global_scope.get_mut_scope(scope_id).new_var("t", false);
                let expr_id = global_scope
                    .get_mut_scope(scope_id)
                    .new_expr(ExprContent::Var(var_id), None);
                var_wires.insert(var_id, wire);
                wire_info.downstream_expr = Some(expr_id);

                if let Some((bundle_var, bundle_var_expr)) = bundle_var_expr {
                    wire_info.bundled_expr =
                        Some((bundle_var, bundle_var_expr, output_wires.len()));
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
        add_to_split(
            port.wire,
            max_split,
            &wire_infos,
            &var_wires,
            &mut splits,
            global_scope.get_scope(scope_id),
        );
    }

    let ids = splits
        .into_iter()
        .map(|split| {
            let expr_id = global_scope
                .get_mut_scope(scope_id)
                .new_expr(ExprContent::List(split.exprs), None);
            let macro_id = global_scope.new_macro(name, expr_id, split.vars, scope_id);
            (macro_id, expr_id)
        })
        .collect::<Vec<_>>();

    let mut next_split = None;
    for &(macro_id, expr_id) in ids.iter().rev() {
        global_scope.get_mut_expr(expr_id, scope_id).wrapper = next_split;
        next_split = Some(macro_id);
    }

    ids.first().unwrap().0
}

fn topo_sort_cells(cells: &OrderMap<String, Cell>) -> Vec<usize> {
    let mut children = vec![Vec::new(); cells.len()];
    let mut incoming = vec![0_usize; cells.len()];

    let connection_cells = cells
        .values()
        .enumerate()
        .flat_map(|(cell_idx, cell)| {
            cell.output_connections()
                .map(move |(_, wire)| (wire, cell_idx))
        })
        .collect::<OrderMap<Wire, usize>>();

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

    assert_eq!(topo.len(), cells.len());
    topo
}

fn consumer_counts(module: &Module) -> OrderMap<Wire, WireInfo> {
    let mut consumer_counts = OrderMap::new();
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
                    input_wires: OrderSet::new(),
                },
            )
        })
        .collect()
}

fn create_inputs(
    wire_infos: &mut OrderMap<Wire, WireInfo>,
    module: &Module,
    local_scope: &mut LocalScope,
) -> OrderMap<VarID, Wire> {
    let mut var_wires = OrderMap::new();

    for (name, port) in module.input_ports() {
        let wire_info = wire_infos.get_mut(&port.wire).unwrap();
        wire_info.split_idx_lb = Some(0);
        wire_info.split_delta = Some(0);

        let var_id = local_scope.new_var(name, true);
        wire_info.input_var = Some(var_id);
        wire_info.expr = Some(local_scope.new_expr(ExprContent::Var(var_id), None));
        var_wires.insert(var_id, port.wire);
    }

    var_wires
}

fn add_reg_macro_prev_output(
    inputs: &mut Vec<(Wire, usize)>,
    cell: &Cell,
    macro_id: MacroID,
    global_scope: &GlobalScope,
    module: &Module,
) {
    if let Some(map_output_to_input_idx) = global_scope
        .macros
        .get(&macro_id)
        .unwrap()
        .map_output_to_input_idx
    {
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

fn compute_split_upper_bounds(max_split: usize, wire_infos: &mut OrderMap<Wire, WireInfo>) {
    if max_split < 2 || wire_infos.is_empty() {
        wire_infos.values_mut().for_each(|info| {
            info.split_idx_ub = info.split_idx_lb;
        });

        return;
    }

    // Build adjacency
    let mut children = OrderMap::new();
    let mut incoming = OrderMap::new();
    for &wire in wire_infos.keys() {
        children.insert(wire, OrderSet::new());
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
    wire_infos: &OrderMap<Wire, WireInfo>,
    var_wires: &OrderMap<VarID, Wire>,
    splits: &mut Vec<Split>,
    local_scope: &LocalScope,
) {
    let info = wire_infos.get(&wire).unwrap();
    let expr_id = info.expr_for_ub_split(target_split_idx);

    splits[target_split_idx].exprs.push(expr_id);
    splits[target_split_idx].output_count += 1;

    for var_id in local_scope.get_expr(expr_id).input_vars(local_scope) {
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
                    add_to_split(
                        *var_wire,
                        split_idx,
                        wire_infos,
                        var_wires,
                        splits,
                        local_scope,
                    );
                }
                break;
            }

            if var_info.split_idx_ub.unwrap() + 1 == split_idx && var_info.bundled_expr.is_some() {
                let (bundle_var, bundle_var_expr, expanded_len) = var_info.bundled_expr.unwrap();
                if prev_input_idx.is_some_and(|prev_input_idx| {
                    prev_input_idx + 1 == split.output_count + expanded_len
                }) {
                    split.exprs.push(bundle_var_expr);
                    split.output_count += expanded_len;
                }

                if let Some(idx) = split.vars.iter().position(|input| *input == bundle_var) {
                    prev_input_idx = Some(idx);
                } else {
                    prev_input_idx = Some(split.vars.len());
                    split.vars.push(bundle_var);
                }
            } else {
                if prev_input_idx.is_some_and(|prev_input_idx| prev_input_idx == split.output_count)
                {
                    split.exprs.push(var_info.expr_for_ub_split(split_idx));
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
