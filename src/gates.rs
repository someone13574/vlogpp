use std::collections::HashMap;

pub const AND_GATE_NAME: &'static str = "$_AND_";
pub fn implement_and(inputs: HashMap<&str, &str>) -> String {
    let a = inputs.get("A").unwrap();
    let b = inputs.get("B").unwrap();

    format!("_AND({a}, {b})")
}

pub const OR_GATE_NAME: &'static str = "$_OR_";
pub fn implement_or(inputs: HashMap<&str, &str>) -> String {
    let a = inputs.get("A").unwrap();
    let b = inputs.get("B").unwrap();

    format!("_OR({a}, {b})")
}

pub const XOR_GATE_NAME: &'static str = "$_XOR_";
pub fn implement_xor(inputs: HashMap<&str, &str>) -> String {
    let a = inputs.get("A").unwrap();
    let b = inputs.get("B").unwrap();

    format!("_XOR({a}, {b})")
}
