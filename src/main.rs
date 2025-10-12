use std::fs;

use crate::{macro_builder::build_module_macro, yosys::Yosys};

pub mod gates;
pub mod macro_builder;
pub mod yosys;

fn main() {
    let design =
        serde_json::from_str::<Yosys>(&fs::read_to_string("design.json").unwrap()).unwrap();
    let module = build_module_macro("ADDER", design.modules.get("adder").unwrap());

    println!("{module}");
}
