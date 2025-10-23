use std::collections::{HashMap, VecDeque};
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
        netlist.postprocess();

        if display {
            netlist.clone().show();
        }
        netlist
    }

    pub fn postprocess(&mut self) {
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

        let mut queue = self.modules.clone().into_iter().collect::<VecDeque<_>>();
        while let Some((module_name, module)) = queue.pop_front() {
            let inputs = module
                .input_ports()
                .map(|(name, port)| (name.clone(), port.wire))
                .collect::<HashMap<_, _>>();
            let outputs = module
                .output_ports()
                .map(|(name, port)| (port.wire, name.clone()))
                .collect::<HashMap<_, _>>();

            let mut next_wire = Wire(0);
            for wire in module
                .cells
                .values()
                .flat_map(|cell| cell.connections.values())
                .chain(module.ports.values().map(|port| &port.wire))
            {
                if wire.0 >= next_wire.0 {
                    next_wire.0 = wire.0 + 1;
                }
            }

            let mut dirty = false;
            for (cell_name, cell) in module.cells {
                if cell.kind == "$_DFF_P_" {
                    let out_wire = cell.connections.get("Q").unwrap();
                    let input_name = format!("{}.i", outputs.get(out_wire).unwrap());
                    let input_wire = *inputs.get(&input_name).unwrap();

                    let cell = self
                        .modules
                        .get_mut(&module_name)
                        .unwrap()
                        .cells
                        .get_mut(&cell_name)
                        .unwrap();
                    cell.port_directions
                        .insert("Q.i".to_string(), PortDirection::Input);
                    cell.connections.insert("Q.i".to_string(), input_wire);
                } else if let Some(instance) = self.modules.get(&cell.kind).cloned() {
                    for (port_name, port) in &instance.ports {
                        let new_port_name = format!("{}..{}", cell_name, port_name);

                        if cell.connections.contains_key(port_name) {
                            assert_eq!(
                                *cell.port_directions.get(port_name).unwrap(),
                                port.direction
                            );

                            if port.direction == PortDirection::Output
                                && instance.ports.contains_key(&format!("{port_name}.i"))
                                && !module.ports.contains_key(&new_port_name)
                            {
                                let wire = cell.connections.get(port_name).unwrap();
                                let module = self.modules.get_mut(&module_name).unwrap();
                                module.ports.insert(
                                    new_port_name,
                                    Port {
                                        direction: PortDirection::Output,
                                        wire: *wire,
                                    },
                                );

                                let cell = module.cells.get_mut(&cell_name).unwrap();
                                cell.port_directions
                                    .insert(port_name.clone(), PortDirection::Output);
                                cell.connections.insert(port_name.clone(), *wire);

                                dirty = true;
                            }
                            continue;
                        }

                        let new_wire = next_wire;
                        next_wire.0 += 1;

                        let module = self.modules.get_mut(&module_name).unwrap();
                        module.ports.insert(
                            new_port_name,
                            Port {
                                direction: port.direction,
                                wire: new_wire,
                            },
                        );

                        let cell = module.cells.get_mut(&cell_name).unwrap();
                        cell.port_directions
                            .insert(port_name.clone(), port.direction);
                        cell.connections.insert(port_name.clone(), new_wire);

                        dirty = true;
                    }

                    let cell = self
                        .modules
                        .get(&module_name)
                        .unwrap()
                        .cells
                        .get(&cell_name)
                        .unwrap()
                        .clone();
                    for (port_name, port) in &instance.ports {
                        if cell.connections.contains_key(port_name) {
                            assert_eq!(
                                *cell.port_directions.get(port_name).unwrap(),
                                port.direction
                            );

                            if port.direction == PortDirection::Input {
                                continue;
                            }

                            let output_wire = cell.connections.get(port_name).unwrap();
                            if let Some(input_wire) =
                                cell.connections.get(&format!("{port_name}.i"))
                            {
                                assert_eq!(
                                    *cell.port_directions.get(&format!("{port_name}.i")).unwrap(),
                                    PortDirection::Input
                                );

                                let module = self.modules.get_mut(&module_name).unwrap();
                                for cell in module.cells.values_mut() {
                                    for (connection_name, connection_wire) in
                                        cell.connections.clone()
                                    {
                                        if connection_wire != *output_wire {
                                            continue;
                                        }

                                        if *cell.port_directions.get(&connection_name).unwrap()
                                            != PortDirection::Input
                                        {
                                            continue;
                                        }

                                        *cell.connections.get_mut(&connection_name).unwrap() =
                                            *input_wire;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if dirty && let Some(callers) = callers.get(&module_name) {
                for caller in callers {
                    queue.push_back((caller.clone(), self.modules.get(caller).unwrap().clone()));
                }
            }
        }
    }

    pub fn show(mut self) {
        self.modules.insert(
            "_DFF_P_".to_string(),
            Module {
                attributes: HashMap::new(),
                ports: [
                    (
                        "Q.i".to_string(),
                        Port {
                            direction: PortDirection::Input,
                            wire: Wire(0),
                        },
                    ),
                    (
                        "D".to_string(),
                        Port {
                            direction: PortDirection::Input,
                            wire: Wire(1),
                        },
                    ),
                    (
                        "C".to_string(),
                        Port {
                            direction: PortDirection::Input,
                            wire: Wire(2),
                        },
                    ),
                    (
                        "Q".to_string(),
                        Port {
                            direction: PortDirection::Output,
                            wire: Wire(3),
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                cells: [(
                    "mux".to_string(),
                    Cell {
                        kind: "$_MUX_".to_string(),
                        port_directions: [
                            ("A".to_string(), PortDirection::Input),
                            ("B".to_string(), PortDirection::Input),
                            ("S".to_string(), PortDirection::Input),
                            ("Y".to_string(), PortDirection::Output),
                        ]
                        .into_iter()
                        .collect(),
                        connections: [
                            ("A".to_string(), Wire(0)),
                            ("B".to_string(), Wire(1)),
                            ("S".to_string(), Wire(2)),
                            ("Y".to_string(), Wire(3)),
                        ]
                        .into_iter()
                        .collect(),
                    },
                )]
                .into_iter()
                .collect(),
            },
        );

        let mut file = File::create("processed.json").unwrap();
        let text = serde_json::to_string_pretty(&self)
            .unwrap()
            .replace("$_DFF_P_", "_DFF_P_");
        file.write_all(text.as_bytes()).unwrap();

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
            .filter(|(_name, port)| port.direction == PortDirection::Input)
    }

    pub fn output_ports(&self) -> impl Iterator<Item = (&String, &Port)> {
        self.ports
            .iter()
            .filter(|(_name, port)| port.direction == PortDirection::Output)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Wire(pub usize);

impl Serialize for Wire {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(std::iter::once(self.0))
    }
}

impl<'de> Deserialize<'de> for Wire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<usize>::deserialize(deserializer)?;
        if vec.len() == 1 {
            Ok(Wire(vec.into_iter().next().unwrap()))
        } else {
            Err(serde::de::Error::custom("Yosys input must be fully split"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub direction: PortDirection,

    #[serde(rename = "bits")]
    pub wire: Wire,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    #[serde(rename = "type")]
    pub kind: String,
    pub port_directions: Map<String, PortDirection>,
    pub connections: Map<String, Wire>,
}

impl Cell {
    pub fn input_connections(&self) -> impl Iterator<Item = (&String, Wire)> {
        self.port_directions
            .iter()
            .filter(|(_, dir)| **dir == PortDirection::Input)
            .map(|(port_name, _)| (port_name, *self.connections.get(port_name).unwrap()))
    }

    pub fn output_connections(&self) -> impl Iterator<Item = (&String, Wire)> {
        self.port_directions
            .iter()
            .filter(|(_, dir)| **dir == PortDirection::Output)
            .map(|(port_name, _)| (port_name, *self.connections.get(port_name).unwrap()))
    }
}
