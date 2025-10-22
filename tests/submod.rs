use std::fs::{self, File};
use std::io::Write;
use std::iter::once;
use std::process::Command;

use vlogpp::lint::lint_directory;
use vlogpp::lut::Lut;
use vlogpp::netlist::Netlist;
use vlogpp::registry::Registry;
use vlogpp::scope::global::GlobalScope;

#[test]
fn test_submods() {
    lint_directory("tests");

    let netlist = Netlist::new("tests/submod.sv", false, &[]);
    let registry = Registry::new()
        .register_lut(Lut::not())
        .register_lut(Lut::or())
        .register_lut(Lut::and())
        .register_lut(Lut::xor())
        .register_lut(Lut::dff_p())
        .add_netlist(netlist);

    let mut global_scope = GlobalScope::new(registry);
    let top = *Registry::top_modules(&mut global_scope).first().unwrap();

    let macro_text = global_scope.emit();
    let top_macro = global_scope.get_macro(top);

    for (a, b, c) in [(106_usize, 22, 1), (211, 165, 0)] {
        let a_bits = format!("{:08b}", a)
            .chars()
            .rev()
            .enumerate()
            .map(|(idx, c)| (format!("a[{idx}]"), c))
            .collect::<Vec<_>>();
        let b_bits = format!("{:08b}", b)
            .chars()
            .rev()
            .enumerate()
            .map(|(idx, c)| (format!("b[{idx}]"), c))
            .collect::<Vec<_>>();
        let c_bit = if c == 1 {
            ("c".to_string(), '1')
        } else {
            ("c".to_string(), '0')
        };

        let mut inputs = a_bits
            .iter()
            .chain(b_bits.iter())
            .chain(once(&c_bit))
            .map(|(name, value)| {
                (
                    top_macro.input_position(name, &global_scope).unwrap(),
                    *value,
                )
            })
            .collect::<Vec<_>>();
        inputs.sort_by_key(|(idx, _)| *idx);

        let output_map = global_scope
            .get_scope(top_macro.scope_id)
            .local()
            .output_names
            .clone()
            .unwrap();
        let mut output_bits = format!("{:09b}", a + b + c)
            .chars()
            .rev()
            .enumerate()
            .map(|(idx, char)| {
                (
                    output_map
                        .iter()
                        .position(|out| out == &format!("out[{idx}]"))
                        .unwrap(),
                    char,
                )
            })
            .collect::<Vec<_>>();
        output_bits.sort_by_key(|(idx, _)| *idx);

        let text = format!(
            "{macro_text}\n{}({})",
            &top_macro.name,
            inputs
                .iter()
                .map(|(_, var)| var.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let mut file = File::create("test.h").unwrap();
        file.write_all(text.as_bytes()).unwrap();

        let status = Command::new("gcc")
            .arg("-E")
            .arg("-P")
            .arg("test.h")
            .arg("-o")
            .arg("test_out.h")
            .status()
            .unwrap();

        assert!(status.success());

        let actual = fs::read_to_string("test_out.h").unwrap();
        assert_eq!(
            actual.replace(" ", ""),
            format!(
                "{}\n",
                output_bits
                    .iter()
                    .map(|(_, out)| out.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        );
    }
}
