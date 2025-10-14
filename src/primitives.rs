use std::iter::once;

use crate::expr::ExprContent;
use crate::global_scope::GlobalScope;
use crate::r#macro::MacroID;

pub fn new_lut_primitive(name: &str, outputs: &[bool], global_scope: &mut GlobalScope) -> MacroID {
    assert!(outputs.len() >= 4 && outputs.len().is_power_of_two());
    let num_inputs = outputs.len().ilog2() as usize;

    let scope_id = global_scope.new_local_scope();
    let vars = ('a'..='z')
        .into_iter()
        .take(num_inputs)
        .map(|x| global_scope.get_mut_scope(scope_id).new_var(&x.to_string()))
        .collect::<Vec<_>>();

    let paste_macro = global_scope.paste_macro(num_inputs + 1, true);
    let prefix = global_scope.get_macro_prefix(name);
    let exprs = once(ExprContent::Text(format!("{prefix}_")))
        .chain(vars.iter().map(|var| ExprContent::Var(*var)))
        .map(|content| global_scope.get_mut_scope(scope_id).new_expr(content, None))
        .collect::<Vec<_>>();

    for (idx, &output) in outputs.iter().enumerate() {
        assert_eq!(
            global_scope.defines.insert(
                format!("{prefix}_{:0len$b}", idx, len = num_inputs),
                if output {
                    "1".to_string()
                } else {
                    "0".to_string()
                },
            ),
            None
        );
    }

    let body = global_scope
        .get_mut_scope(scope_id)
        .new_expr(ExprContent::List(exprs), Some(paste_macro));
    let macro_id = global_scope.new_macro(&prefix, body, vars, scope_id);
    assert_eq!(
        global_scope.modules.insert(name.to_string(), macro_id),
        None,
        "Module `{name}` redefined."
    );

    macro_id
}
