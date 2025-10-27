use vlogpp::lint::lint_directory;
use vlogpp::netlist::Netlist;
use vlogpp::sat::Combinational;

fn main() {
    lint_directory("circuits");

    let netlist = Netlist::new("circuits/adder.sv", true, &[]);
    let mut circuit = Combinational::new(netlist, None);
    circuit.generate();
}
