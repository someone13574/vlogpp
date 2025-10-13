use std::fs;

use crate::{macro_builder::build_module_macro, names::GlobalNames, yosys::Yosys};

pub mod gates;
pub mod macro_builder;
pub mod names;
pub mod yosys;

fn main() {
    let mut global_names = GlobalNames::new();

    let design =
        serde_json::from_str::<Yosys>(&fs::read_to_string("design.json").unwrap()).unwrap();
    let module = build_module_macro(
        "adder",
        design.modules.get("adder").unwrap(),
        &mut global_names,
    );

    println!("{module}");
}
