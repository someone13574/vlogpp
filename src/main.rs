use std::fs;

use crate::{scope::GlobalScope, yosys::Yosys};

pub mod expr;
pub mod gates;
pub mod r#macro;
pub mod macro_builder;
pub mod names;
pub mod primitives;
pub mod scope;
pub mod yosys;

fn main() {
    let yosys = serde_json::from_str::<Yosys>(&fs::read_to_string("design.json").unwrap()).unwrap();
    let mut global_scope = GlobalScope::new(yosys);
    global_scope.get_paste_macro(5, true);
    global_scope.get_paste_macro(2, false);

    println!("{}", global_scope);
}
