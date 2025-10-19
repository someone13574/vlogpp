use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

use ordermap::OrderMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Netlist {
    pub creator: String,
    pub modules: HashMap<String, Module>,
}

impl Netlist {
    pub fn new<P: AsRef<Path>>(file: P, display: bool, top_params: &[(&str, &str, &str)]) -> Self {
        let display_command = if display {
            "show -stretch -format ps -viewer evince;"
        } else {
            ""
        };

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
            {}
            write_json design.json", file.as_ref().display(), display_command};

        Command::new("yosys")
            .arg("-p")
            .arg(commands)
            .status()
            .expect("Yosys netlist generation failed");

        let buffer = BufReader::new(File::open("design.json").unwrap());
        serde_json::from_reader(buffer).unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Module {
    pub attributes: HashMap<String, String>,
    pub ports: OrderMap<String, Port>,
    pub cells: OrderMap<String, Cell>,
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

#[derive(Debug, Clone, Deserialize)]
pub struct Port {
    pub direction: PortDirection,

    #[serde(rename = "bits")]
    pub wire: Wire,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cell {
    #[serde(rename = "type")]
    pub kind: String,
    pub port_directions: OrderMap<String, PortDirection>,
    pub connections: OrderMap<String, Wire>,
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
