use std::collections::{HashMap, VecDeque};

use ordermap::OrderMap;

use crate::expr::{ExprContent, ExprID};
use crate::global_scope::GlobalScope;
use crate::r#macro::MacroID;
use crate::yosys::{Cell, Module, Wire};

pub fn create_module(name: &str, module: &Module, global_scope: &mut GlobalScope) -> MacroID {
    let scope_id = global_scope.new_local_scope();

    let topo = topo_sort_cells(&module.cells);
    let inputs = module
        .input_ports()
        .map(|(var_name, port)| {
            (
                global_scope.get_mut_scope(scope_id).new_var(var_name, true),
                port.wire,
            )
        })
        .collect::<Vec<_>>();
    global_scope.get_mut_scope(scope_id).output_names = Some(
        module
            .output_ports()
            .map(|(name, _)| name.to_string())
            .collect(),
    );

    // let consumer_counts = consumer_counts(module);
    let mut wire_exprs = inputs
        .iter()
        .map(|(var_id, wire)| {
            (
                *wire,
                global_scope
                    .get_mut_scope(scope_id)
                    .new_expr(ExprContent::Var(*var_id), None),
            )
        })
        .collect::<HashMap<Wire, ExprID>>();

    for &cell_idx in &topo {
        let (_cell_name, cell) = module.cells.get_index(cell_idx).unwrap();
        let call_module = global_scope.get_module(&cell.kind).unwrap();

        assert_eq!(
            cell.output_connections().count(),
            1,
            "Only single output is supported for now"
        );

        let mut inputs = cell
            .input_connections()
            .map(|(name, wire)| (name, *wire_exprs.get(&wire).unwrap()))
            .collect::<Vec<_>>();
        inputs.sort_by_key(|(name, _)| {
            global_scope
                .macros
                .get(&call_module)
                .unwrap()
                .input_position(name, global_scope)
                .unwrap()
        });

        let content = ExprContent::List(inputs.into_iter().map(|(_, expr)| expr).collect());
        let expr_id = global_scope
            .get_mut_scope(scope_id)
            .new_expr(content, Some(call_module));
        wire_exprs.insert(cell.output_connections().next().unwrap().1, expr_id);
    }

    let content = ExprContent::List(
        module
            .output_ports()
            .map(|(_, port)| *wire_exprs.get(&port.wire).unwrap())
            .collect(),
    );
    let expr = global_scope.get_mut_scope(scope_id).new_expr(content, None);
    global_scope.new_macro(
        name,
        expr,
        inputs.iter().map(|(input, _)| *input).collect(),
        scope_id,
    )
}

// fn consumer_counts(module: &Module) -> HashMap<Wire, usize> {
//     let mut counts = HashMap::new();
//     for wire in module.output_ports().map(|(_name, port)| port.wire).chain(
//         module
//             .cells
//             .values()
//             .flat_map(|cell| cell.input_connections().map(|(_, wire)| wire)),
//     ) {
//         counts
//             .entry(wire)
//             .and_modify(|count| *count += 1)
//             .or_insert(1);
//     }

//     counts
// }

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
        .collect::<HashMap<Wire, usize>>();

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
