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

    pub split_delta: Option<usize>,
    pub split_idx_lb: Option<usize>,
    pub split_idx_ub: Option<usize>,

    pub consumers: usize,
    pub input_wires: OrderSet<Wire>,
}

impl WireInfo {
    fn downstream_expr(&self) -> ExprID {
        self.downstream_expr.or(self.expr).unwrap()
    }

    fn downstream_split_idx_lb(&self) -> usize {
        self.split_idx_lb.unwrap() + self.split_delta.unwrap()
    }
}

#[derive(Clone)]
struct Split {
    pub exprs: Vec<ExprID>,
    pub vars: Vec<VarID>,
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
        let call_module = global_scope.get_module(&cell.kind).unwrap();

        assert_eq!(
            cell.output_connections().count(),
            1,
            "Only single output is supported for now"
        );

        let mut inputs = cell
            .input_connections()
            .map(|(name, wire)| (name, wire, wire_infos.get(&wire).unwrap().downstream_expr()))
            .collect::<Vec<_>>();
        inputs.sort_by_key(|(name, _, _)| {
            global_scope
                .macros
                .get(&call_module)
                .unwrap()
                .input_position(name, global_scope)
                .unwrap()
        });

        let split_idx_lb = inputs
            .iter()
            .map(|(_, wire, _)| wire_infos.get(wire).unwrap().downstream_split_idx_lb())
            .max()
            .unwrap();

        let content = ExprContent::List(inputs.iter().map(|(_, _, expr)| *expr).collect());
        let expr_id = global_scope
            .get_mut_scope(scope_id)
            .new_expr(content, Some(call_module));

        let wire = cell.output_connections().next().unwrap().1;
        let wire_info = wire_infos.get_mut(&wire).unwrap();
        wire_info.expr = Some(expr_id);
        wire_info.split_idx_lb = Some(split_idx_lb);
        wire_info
            .input_wires
            .extend(inputs.iter().map(|(_, wire, _)| wire));

        // Will need to get all consumers in bundle for multi-output
        if wire_info.consumers > 1 {
            let temp_var = global_scope.get_mut_scope(scope_id).new_var("t", false);
            let expr_id = global_scope
                .get_mut_scope(scope_id)
                .new_expr(ExprContent::Var(temp_var), None);
            var_wires.insert(temp_var, wire);

            wire_info.downstream_expr = Some(expr_id);
            wire_info.split_delta = Some(1);
        } else {
            wire_info.split_delta = Some(0);
        }
    }

    let max_split = wire_infos
        .values()
        .map(|info| info.split_idx_lb.unwrap())
        .max()
        .unwrap_or_default();
    compute_split_upper_bounds(max_split, &mut wire_infos);

    let mut splits = vec![
        Split {
            exprs: Vec::new(),
            vars: Vec::new()
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

    ids.iter().next().unwrap().0
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
            wire_info.split_idx_ub = Some(max_split);
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
    let expr_id = if info.split_idx_ub.unwrap() == target_split_idx {
        info.expr.unwrap()
    } else {
        info.downstream_expr()
    };

    splits[target_split_idx].exprs.push(expr_id);

    for var_id in local_scope
        .get_expr(expr_id)
        .input_vars(local_scope)
        .into_iter()
    {
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
                if prev_input_idx.is_some_and(|prev_input_idx| prev_input_idx == split.exprs.len())
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

            if prev_input_idx.is_some_and(|prev_input_idx| prev_input_idx == split.exprs.len()) {
                split.exprs.push(var_info.downstream_expr());
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
