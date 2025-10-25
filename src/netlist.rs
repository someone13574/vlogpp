use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use std::process::Command;

use ordermap::OrderMap;
use serde::{Deserialize, Serialize};

use crate::Map;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Netlist {
    pub creator: String,
    pub modules: OrderMap<String, Module>,
}

impl Netlist {
    pub fn new<P: AsRef<Path>>(file: P, display: bool, top_params: &[(&str, &str, &str)]) -> Self {
        let params = top_params
            .iter()
            .map(|(k, v, module)| format!("chparam -set {k} {v} {module};"))
            .collect::<Vec<_>>()
            .join("");

        let commands = indoc::formatdoc! {"
            read_verilog -sv {};
            {params}
            hierarchy -check -auto-top;
            proc;; memory;; fsm;; wreduce;; opt -full;;
            techmap;; opt -full;;
            splitnets -ports;; expose -dff -cut;; opt -full;;
            clean -purge;
            write_json design.json", file.as_ref().display()};

        let status = Command::new("yosys")
            .arg("-p")
            .arg(commands)
            .status()
            .expect("Yosys netlist generation failed");
        assert!(status.success());

        let buffer = BufReader::new(File::open("design.json").unwrap());
        let mut netlist: Netlist = serde_json::from_reader(buffer).unwrap();
        netlist.remove_flip_flops();

        if display {
            netlist.show();
        }
        netlist
    }

    fn remove_flip_flops(&mut self) {
        let mut callers = HashMap::new();
        for (module_name, module) in self.modules.iter() {
            for cell in module.cells.values() {
                if self.modules.contains_key(&cell.kind) {
                    callers
                        .entry(cell.kind.clone())
                        .and_modify(|callers: &mut Vec<String>| {
                            callers.push(module_name.clone());
                        })
                        .or_insert(vec![module_name.clone()]);
                }
            }
        }

        let mut queue = self.modules.keys().cloned().collect::<VecDeque<_>>();
        let mut removed_ports = HashMap::new();

        while let Some(module_name) = queue.pop_back() {
            let mut next_wire_id = 0;
            for wire in self
                .modules
                .get(&module_name)
                .unwrap()
                .cells
                .values()
                .flat_map(|cell| cell.connections.values())
                .chain(
                    self.modules
                        .get(&module_name)
                        .unwrap()
                        .ports
                        .values()
                        .map(|port| &port.wire),
                )
            {
                let Wire::Wire(wire) = *wire else { continue };

                if wire >= next_wire_id {
                    next_wire_id = wire + 1;
                }
            }

            let mut interface_modified = false;
            for (cell_name, cell_clone) in self.modules.get(&module_name).unwrap().cells.clone() {
                if cell_clone.kind == "$_DFF_P_" {
                    // Remove flip-flops by using the D port directly
                    let module = self.modules.get_mut(&module_name).unwrap();
                    let data_wire = *cell_clone.connections.get("D").unwrap();
                    let output_wire = *cell_clone.connections.get("Q").unwrap();

                    for cell in module.cells.values() {
                        for (connection_name, connection) in cell.connections.iter() {
                            // Flip-flops shouldn't connect to anything other than ports
                            let port_direction = cell.port_dirs.get(connection_name).unwrap();
                            assert!(
                                *connection != output_wire || *port_direction != PortDir::Input
                            );
                        }
                    }

                    for port in module.ports.values_mut() {
                        if port.wire == output_wire && port.dir == PortDir::Output {
                            port.wire = data_wire;
                        }
                    }

                    module.cells.remove(&cell_name).unwrap();
                } else if let Some(submod_clone) = self.modules.get(&cell_clone.kind).cloned() {
                    let module = self.modules.get_mut(&module_name).unwrap();
                    let cell = module.cells.get_mut(&cell_name).unwrap();

                    // Remove removed ports
                    if let Some(removed_ports) = removed_ports.get(&cell_clone.kind) {
                        for removed_port in removed_ports {
                            cell.port_dirs.remove(removed_port).unwrap();
                            cell.connections.remove(removed_port).unwrap();
                        }
                    }

                    // Add missing ports
                    for (submod_port_name, submod_port) in &submod_clone.ports {
                        if let Some((_, cell_port)) = cell
                            .output_connections()
                            .find(|(name, _)| *name == submod_port_name)
                        {
                            // Port exists on cell, but doesn't output to the module
                            if submod_port.dir == PortDir::Output
                                && submod_clone
                                    .ports
                                    .contains_key(&format!("{submod_port_name}.i"))
                            {
                                let inserted = module
                                    .ports
                                    .insert(
                                        format!("{cell_name}..{submod_port_name}"),
                                        Port {
                                            dir: submod_port.dir,
                                            wire: cell_port,
                                        },
                                    )
                                    .is_none();

                                interface_modified = interface_modified || inserted;
                            }
                        } else if !cell.connections.contains_key(submod_port_name) {
                            // Port doesn't exist for the module or the cell
                            let new_wire = Wire::Wire(next_wire_id);
                            next_wire_id += 1;

                            assert!(
                                cell.connections
                                    .insert(submod_port_name.clone(), new_wire)
                                    .is_none()
                            );
                            assert!(
                                cell.port_dirs
                                    .insert(submod_port_name.clone(), submod_port.dir)
                                    .is_none()
                            );
                            assert!(
                                module
                                    .ports
                                    .insert(
                                        format!("{cell_name}..{submod_port_name}"),
                                        Port {
                                            dir: submod_port.dir,
                                            wire: new_wire
                                        }
                                    )
                                    .is_none()
                            );
                            interface_modified = true;
                        }
                    }

                    // Reconnect cells using stateful module output to use the matching input
                    let cell_clone = module.cells.get(&cell_name).unwrap().clone();
                    for (port_name, port_dir) in cell_clone.port_dirs.clone() {
                        if port_dir != PortDir::Output {
                            continue;
                        }

                        let Some(cell_out_wire) = cell_clone.connections.get(&port_name).copied()
                        else {
                            continue;
                        };

                        let Some(matching_input) = cell_clone
                            .connections
                            .get(&format!("{port_name}.i"))
                            .copied()
                        else {
                            continue;
                        };

                        for cell in module.cells.values_mut() {
                            for (conn_name, conn) in cell.connections.clone() {
                                if conn != cell_out_wire {
                                    continue;
                                }

                                if *cell.port_dirs.get(&conn_name).unwrap() != PortDir::Input {
                                    continue;
                                }

                                *cell.connections.get_mut(&conn_name).unwrap() = matching_input;
                            }
                        }
                    }
                }
            }

            // Remove unused ports (clk)
            let module = self.modules.get_mut(&module_name).unwrap();
            let mut inputs_used = module
                .input_ports()
                .map(|(name, port)| (port.wire, (false, name.clone())))
                .collect::<HashMap<_, _>>();
            for cell in module.cells.values() {
                for (_, wire) in cell.input_connections() {
                    if let Some((used, _)) = inputs_used.get_mut(&wire) {
                        *used = true;
                    }
                }
            }

            for (_, (used, port_name)) in inputs_used {
                if !used {
                    module.ports.remove(&port_name).unwrap();
                    removed_ports
                        .entry(module_name.clone())
                        .and_modify(|set: &mut HashSet<String>| {
                            set.insert(port_name.clone());
                        })
                        .or_insert_with(|| {
                            let mut set = HashSet::new();
                            set.insert(port_name.clone());
                            set
                        });
                    interface_modified = true;
                }
            }

            // Add callers to queue
            if interface_modified && let Some(callers) = callers.get(&module_name) {
                for caller in callers {
                    if queue.contains(caller) {
                        continue;
                    }

                    queue.push_back(caller.clone());
                }
            }
        }
    }

    pub fn show(&self) {
        let mut file = File::create("processed.json").unwrap();
        file.write_all(&serde_json::to_vec_pretty(self).unwrap())
            .unwrap();

        let status = Command::new("yosys")
            .arg("-p")
            .arg("read_json processed.json; show -stretch -format ps -viewer evince;")
            .status()
            .unwrap();
        assert!(status.success());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub attributes: HashMap<String, String>,
    pub ports: Map<String, Port>,
    pub cells: Map<String, Cell>,
}

impl Module {
    pub fn input_ports(&self) -> impl Iterator<Item = (&String, &Port)> {
        self.ports
            .iter()
            .filter(|(_name, port)| port.dir == PortDir::Input)
    }

    pub fn output_ports(&self) -> impl Iterator<Item = (&String, &Port)> {
        self.ports
            .iter()
            .filter(|(_name, port)| port.dir == PortDir::Output)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Wire {
    Wire(usize),
    Const(bool),
}

impl Serialize for Wire {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Wire::Wire(id) => serializer.collect_seq(std::iter::once(*id)),
            Wire::Const(constant) => {
                serializer.collect_seq(std::iter::once(if *constant { "1" } else { "0" }))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Wire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Item {
            Num(usize),
            Str(String),
        }

        let vec = Vec::<Item>::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        if vec.len() != 1 {
            return Err(serde::de::Error::custom("Yosys input must be fully split"));
        }

        match vec.into_iter().next().unwrap() {
            Item::Num(n) => Ok(Wire::Wire(n)),
            Item::Str(s) => {
                match s.as_str() {
                    "0" => Ok(Wire::Const(false)),
                    "1" => Ok(Wire::Const(true)),
                    other => {
                        Err(serde::de::Error::custom(format!(
                            "unexpected string for constant: {}",
                            other
                        )))
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    #[serde(rename = "direction")]
    pub dir: PortDir,

    #[serde(rename = "bits")]
    pub wire: Wire,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortDir {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "port_directions")]
    pub port_dirs: Map<String, PortDir>,
    pub connections: Map<String, Wire>,
}

impl Cell {
    pub fn input_connections(&self) -> impl Iterator<Item = (&String, Wire)> {
        self.port_dirs
            .iter()
            .filter(|(_, dir)| **dir == PortDir::Input)
            .map(|(port_name, _)| (port_name, *self.connections.get(port_name).unwrap()))
    }

    pub fn output_connections(&self) -> impl Iterator<Item = (&String, Wire)> {
        self.port_dirs
            .iter()
            .filter(|(_, dir)| **dir == PortDir::Output)
            .map(|(port_name, _)| (port_name, *self.connections.get(port_name).unwrap()))
    }
}
