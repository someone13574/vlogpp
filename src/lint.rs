use std::collections::HashMap;
use std::fs;
use std::path::Path;

use codespan_reporting::diagnostic::Label;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use sv_parser::parse_sv_str;
use svlint::config::{Config, ConfigOption, ConfigSyntaxRules, ConfigTextRules};
use svlint::linter::{LintFailed, Linter, TextRuleEvent};
use walkdir::WalkDir;

pub fn lint_directory<P: AsRef<Path>>(path: P) {
    let all_config = Config::new().enable_all();
    let mut linter = Linter::new(Config {
        option: ConfigOption {
            ..all_config.option
        },
        textrules: ConfigTextRules {
            header_copyright: false,
            ..all_config.textrules
        },
        syntaxrules: ConfigSyntaxRules {
            style_commaleading: false,
            style_keyword_1space: false,
            module_identifier_matches_filename: false,
            module_ansi_forbidden: false,
            uppercamelcase_module: false,
            prefix_module: false,
            re_forbidden_module_ansi: false,
            re_forbidden_port_input: false,
            re_forbidden_port_output: false,
            re_forbidden_parameter: false,
            re_forbidden_instance: false,
            prefix_input: false,
            prefix_output: false,
            prefix_instance: false,
            keyword_forbidden_logic: false,
            keyword_forbidden_always_ff: false,
            sequential_block_in_always_ff: false,
            ..all_config.syntaxrules
        },
    });

    let mut files = Vec::new();
    for entry in WalkDir::new(&path)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.metadata().unwrap().is_file()
            && ["v", "sv", "vh", "svh"]
                .contains(&entry.path().extension().unwrap().to_str().unwrap())
        {
            files.push(entry.path().to_path_buf());
        }
    }

    let mut pass = true;
    let mut diag_files = SimpleFiles::new();

    for file in files.iter().map(|file| file.as_path()) {
        let _ = linter.textrules_check(TextRuleEvent::StartOfFile, file, &0);

        let text = fs::read_to_string(file).unwrap();
        let mut beg = 0;

        let file_id = diag_files.add(file.to_str().unwrap(), text.clone());

        for line in text.split_inclusive('\n') {
            for failed in linter.textrules_check(
                TextRuleEvent::Line(line.trim_end_matches(['\n', '\r'])),
                file,
                &beg,
            ) {
                print_lintfailed(failed, &diag_files, file_id);
                pass = false;
            }

            beg += line.len();
        }

        match parse_sv_str(&text, file, &HashMap::new(), &[file], true, false) {
            Ok((syntax_tree, _)) => {
                for node in syntax_tree.into_iter().event() {
                    for failed in linter.syntaxrules_check(&syntax_tree, &node) {
                        print_lintfailed(failed, &diag_files, file_id);
                        pass = false;
                    }
                }
            }
            Err(x) => {
                println!("{x:?}");
                pass = false;
            }
        }
    }

    if !pass {
        panic!("Linting failed");
    }
}

fn print_lintfailed(failed: LintFailed, files: &SimpleFiles<&str, String>, file_id: usize) {
    let diagnostic = codespan_reporting::diagnostic::Diagnostic::error()
        .with_code(&failed.name)
        .with_message(&failed.reason)
        .with_label(
            Label::primary(file_id, failed.beg..failed.beg + failed.len).with_message(&failed.hint),
        );
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    term::emit_to_write_style(&mut writer.lock(), &config, files, &diagnostic).unwrap();
}
