use vlogpp::lint::lint_directory;
use vlogpp::lut::Lut;
use vlogpp::netlist::Netlist;
use vlogpp::registry::Registry;
use vlogpp::scope::global::GlobalScope;

fn main() {
    lint_directory("circuits");

    let netlist = Netlist::new("circuits/stateful.sv", true, &[]);
    let registry = Registry::new()
        .register_lut(Lut::not())
        .register_lut(Lut::or())
        .register_lut(Lut::and())
        .register_lut(Lut::xor())
        .register_lut(Lut::dff_p())
        .add_netlist(netlist);

    let mut global_scope = GlobalScope::new(registry);
    Registry::top_modules(&mut global_scope);
    global_scope.variadicify_macros(2);

    println!("{global_scope}");
}
