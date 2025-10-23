use std::fs::{self, File};
use std::io::Write;
use std::iter::once;
use std::process::Command;

use vlogpp::lint::lint_directory;
use vlogpp::netlist::Netlist;
use vlogpp::registry::Registry;
use vlogpp::scope::global::GlobalScope;

#[test]
fn test_submod_state() {
    lint_directory("tests");

    let netlist = Netlist::new("tests/submod_state.sv", false, &[]);
    let registry = Registry::default().add_netlist(netlist);

    let mut global_scope = GlobalScope::new(registry);
    let top = *Registry::top_modules(&mut global_scope).first().unwrap();
    global_scope.variadicify_macros(2);

    let macro_text = global_scope.emit();
    let top_macro = global_scope.get_macro(top);

    for (sub_in, main_in) in [(4_usize, 79_usize), (12, 123)] {
        let sub_in_bits = format!("{:04b}", sub_in)
            .chars()
            .rev()
            .enumerate()
            .map(|(idx, c)| (format!("sub..sub_cnt[{idx}].i"), c))
            .collect::<Vec<_>>();
        let main_in_bits = format!("{:08b}", main_in)
            .chars()
            .rev()
            .enumerate()
            .map(|(idx, c)| (format!("cnt[{idx}].i"), c))
            .collect::<Vec<_>>();

        let mut inputs = sub_in_bits
            .iter()
            .chain(main_in_bits.iter())
            .chain(once(&("clk".to_string(), '1')))
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
        let mut output_bits = format!("{:08b}", (sub_in + main_in) % 256)
            .chars()
            .rev()
            .enumerate()
            .map(|(idx, char)| (format!("cnt[{idx}]"), char))
            .chain(
                format!("{:04b}", (sub_in + 1) % 16)
                    .chars()
                    .rev()
                    .enumerate()
                    .map(|(idx, char)| (format!("sub..sub_cnt[{idx}]"), char)),
            )
            .map(|(name, char)| {
                (
                    output_map.iter().position(|out| out == &name).unwrap(),
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
