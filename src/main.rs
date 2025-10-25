use vlogpp::lint::lint_directory;
use vlogpp::netlist::Netlist;
use vlogpp::registry::Registry;
use vlogpp::scope::global::GlobalScope;

fn main() {
    lint_directory("circuits");

    let netlist = Netlist::new("circuits/adder.sv", true, &[]);
    let registry = Registry::default().add_netlist(netlist);

    let mut global_scope = GlobalScope::new(registry);
    Registry::module(&mut global_scope, "adder").unwrap();
    global_scope.variadicify_macros(1);

    // Registry::repeat_macro(&mut global_scope, top, "cont");
    // Registry::eval_multiplier(&mut global_scope, 5);
    println!("{global_scope}");
}
