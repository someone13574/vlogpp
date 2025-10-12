use std::collections::HashMap;

use ordermap::OrderMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Yosys {
    pub creator: String,
    pub modules: HashMap<String, Module>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Module {
    pub attributes: HashMap<String, String>,
    pub ports: OrderMap<String, Port>,
    pub cells: HashMap<String, Cell>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Port {
    pub direction: PortDirection,
    pub bits: Vec<usize>,
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
    pub port_directions: HashMap<String, PortDirection>,
    pub connections: HashMap<String, Vec<usize>>,
}
