use vlogpp::lint::lint_directory;
use vlogpp::netlist::Netlist;
use vlogpp::registry::Registry;
use vlogpp::scope::global::GlobalScope;

fn main() {
    lint_directory("circuits");

    let netlist = Netlist::new("circuits/stateful.sv", false, &[]);
    let registry = Registry::default().add_netlist(netlist);

    let mut global_scope = GlobalScope::new(registry);
    Registry::top_modules(&mut global_scope);
    global_scope.variadicify_macros(2);

    println!("{global_scope}");
}
