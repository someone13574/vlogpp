use std::fs;

use crate::global_scope::GlobalScope;
use crate::primitives::new_lut_primitive;
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
    new_lut_primitive("or", &[false, true, true, true], &mut global_scope);
    new_lut_primitive("and", &[false, true, true, false], &mut global_scope);
    new_lut_primitive("xor", &[false, true, true, false], &mut global_scope);

    println!("{}", global_scope);
}
