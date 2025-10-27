pub mod expr;
pub mod lint;
pub mod lut;
pub mod r#macro;
pub mod module;
pub mod netlist;
pub mod registry;
pub mod scope;

#[cfg(feature = "sat")]
pub mod sat;

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
const PREFIX_SEP: &str = "";
