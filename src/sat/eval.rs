use std::collections::{HashMap, VecDeque};

use crate::lut::Lut;
use crate::netlist::{Netlist, Wire};

pub fn eval_cell(
    netlist: &Netlist,
    cell_kind: &str,
    inputs: HashMap<String, bool>,
) -> HashMap<String, bool> {
    match cell_kind {
        "$_AND_" => eval_lut(inputs, Lut::and()),
        "$_OR_" => eval_lut(inputs, Lut::or()),
        "$_XOR_" => eval_lut(inputs, Lut::xor()),
        _ => eval_module(netlist, cell_kind, inputs),
    }
}

pub fn eval_module(
    netlist: &Netlist,
    module_name: &str,
    inputs: HashMap<String, bool>,
) -> HashMap<String, bool> {
    let module = netlist
        .modules
        .get(module_name)
        .expect(&format!("Unknown module `{module_name}`"));

    let mut wire_values = HashMap::new();
    for (input_name, port) in module.input_ports() {
        match port.wire {
            Wire::Wire(_) => {
                wire_values.insert(port.wire, *inputs.get(input_name).unwrap());
            }
            Wire::Const(value) => {
                wire_values.insert(port.wire, value);
            }
        }
    }

    let mut cell_queue = module.cells.iter().collect::<VecDeque<_>>();
    let mut failed = 0;
    'outer: while let Some((cell_name, cell)) = cell_queue.pop_front() {
        let mut cell_inputs = HashMap::new();
        for (input_name, wire) in cell.input_connections() {
            if let Some(&value) = wire_values.get(&wire) {
                cell_inputs.insert(input_name.clone(), value);
                continue;
            }

            cell_queue.push_back((cell_name, cell));
            failed += 1;
            assert!(failed < cell_queue.len() * 2);
            continue 'outer;
        }

        failed = 0;
        let outputs = eval_cell(netlist, &cell.kind, cell_inputs);
        for (output_name, wire) in cell.output_connections() {
            wire_values.insert(wire, *outputs.get(output_name).unwrap());
        }
    }

    let mut outputs = HashMap::new();
    for (output_name, port) in module.output_ports() {
        outputs.insert(output_name.clone(), *wire_values.get(&port.wire).unwrap());
    }

    outputs
}

pub fn eval_lut(inputs: HashMap<String, bool>, lut: Lut) -> HashMap<String, bool> {
    let mut out_idx = 0;
    for (idx, input_name) in lut.input_names.iter().enumerate() {
        out_idx += if *inputs.get(*input_name).unwrap() {
            1 << idx
        } else {
            0
        };
    }

    let mut outputs = HashMap::new();
    outputs.insert(
        lut.output_name.to_string(),
        *lut.outputs.get(out_idx).unwrap(),
    );
    outputs
}
