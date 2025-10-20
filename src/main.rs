use crate::lut::Lut;
use crate::netlist::Netlist;
use crate::registry::Registry;
use crate::scope::global::GlobalScope;

pub mod expr;
pub mod lut;
pub mod r#macro;
pub mod module;
pub mod netlist;
pub mod registry;
pub mod scope;

#[cfg(not(feature = "obfuscate"))]
pub type Map<K, V> = ordermap::OrderMap<K, V>;
#[cfg(not(feature = "obfuscate"))]
pub type Set<T> = ordermap::OrderSet<T>;
#[cfg(not(feature = "obfuscate"))]
const PREFIX_SEP: &str = "_";

#[cfg(feature = "obfuscate")]
pub type Map<K, V> = std::collections::HashMap<K, V>;
#[cfg(feature = "obfuscate")]
pub type Set<T> = std::collections::HashSet<T>;
#[cfg(feature = "obfuscate")]
const PREFIX_SEP: &'static str = "";

fn main() {
    let netlist = Netlist::new("circuits/counter.v", true, &[]);
    let registry = Registry::new()
        .register_lut(Lut::not())
        .register_lut(Lut::or())
        .register_lut(Lut::and())
        .register_lut(Lut::xor())
        .register_lut(Lut::dff_p())
        .add_netlist(netlist);

    let mut global_scope = GlobalScope::new(registry);
    Registry::top_module(&mut global_scope).unwrap();

    println!("{global_scope}");
}
