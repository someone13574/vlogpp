// use crate::global_scope::GlobalScope;
// use crate::lut::new_lut_macro;
// use crate::netlist::Netlist;

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

fn main() {
    let netlist = Netlist::new(
        "circuits/vlogpp_repeat_dec.v",
        true,
        &[("WIDTH", "4", "vlogpp_repeat_dec")],
    );
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
