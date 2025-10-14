use std::fs;

use crate::global_scope::GlobalScope;
use crate::yosys::Yosys;

pub mod expr;
pub mod global_scope;
pub mod local_scope;
pub mod r#macro;
pub mod primitives;
pub mod yosys;

fn main() {
    let yosys = serde_json::from_str::<Yosys>(&fs::read_to_string("design.json").unwrap()).unwrap();
    let mut global_scope = GlobalScope::new(yosys);
    global_scope.paste_macro(5, true);
    global_scope.paste_macro(2, false);

    println!("{}", global_scope);
}
