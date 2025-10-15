use std::fs;

use crate::global_scope::GlobalScope;
use crate::primitive::new_lut_primitive;
use crate::yosys::Yosys;

pub mod expr;
pub mod global_scope;
pub mod local_scope;
pub mod r#macro;
pub mod module;
pub mod primitive;
pub mod yosys;

fn main() {
    let yosys = serde_json::from_str::<Yosys>(&fs::read_to_string("design.json").unwrap()).unwrap();
    let mut global_scope = GlobalScope::new(yosys);
    new_lut_primitive(
        "$_OR_",
        &["A", "B"],
        "Y",
        &[false, true, true, true],
        None,
        &mut global_scope,
    );
    new_lut_primitive(
        "$_AND_",
        &["A", "B"],
        "Y",
        &[false, false, false, true],
        None,
        &mut global_scope,
    );
    new_lut_primitive(
        "$_XOR_",
        &["A", "B"],
        "Y",
        &[false, true, true, false],
        None,
        &mut global_scope,
    );
    new_lut_primitive(
        "$_DFF_P_",
        &["C", "D", "pQ"],
        "Q",
        &[false, true, false, true, false, false, true, true],
        Some(2),
        &mut global_scope,
    );
    new_lut_primitive(
        "$_NOT_",
        &["A"],
        "Y",
        &[true, false],
        None,
        &mut global_scope,
    );
    global_scope.get_root_module().unwrap();

    println!("{}", global_scope);
}
