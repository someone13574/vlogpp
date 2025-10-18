use crate::expr::ExprContent;
use crate::global_scope::GlobalScope;
use crate::r#macro::MacroID;

pub struct RecursionMacros {
    counter_tail: Option<String>,
    is_done_true: Option<String>,
    is_done_false: Option<String>,

    decrement: Option<MacroID>,
    when_done: Option<MacroID>,
}

impl RecursionMacros {
    pub fn new() -> Self {
        Self {
            counter_tail: None,
            is_done_true: None,
            is_done_false: None,
            decrement: None,
            when_done: None,
        }
    }

    pub fn decrement(&mut self, global_scope: &mut GlobalScope) -> MacroID {
        if let Some(decrement) = self.decrement {
            return decrement;
        }

        // Main macro
        let scope_id = global_scope.new_local_scope();
        let paste_macro = global_scope.paste_macro(2, true);
        let prefix = global_scope.get_alias("DECREMENT", true);

        let prefix_expr = global_scope
            .get_mut_scope(scope_id)
            .new_expr(ExprContent::Text(format!("{prefix}_")), None);
        let var = global_scope.get_mut_scope(scope_id).new_var("count", false);
        let var_expr = global_scope
            .get_mut_scope(scope_id)
            .new_expr(ExprContent::Var(var), None);

        let decrement_expr = global_scope.get_mut_scope(scope_id).new_expr(
            ExprContent::List(vec![prefix_expr, var_expr]),
            Some((paste_macro, None)),
        );
        self.decrement = Some(global_scope.new_macro(&prefix, decrement_expr, vec![var], scope_id));

        // Lookup macros
        let counter_tail_keyword = self.counter_tail(global_scope);

        for (key, value) in [
            ("0", ""),
            ("1", "0"),
            (&counter_tail_keyword, &counter_tail_keyword),
        ] {
            global_scope.new_define(format!("{prefix}_{key}"), value.to_string());
        }

        self.decrement.unwrap()
    }

    pub fn when_done(&mut self, global_scope: &mut GlobalScope) -> MacroID {
        if let Some(when_done) = self.when_done {
            return when_done;
        }

        let scope_id = global_scope.new_local_scope();
        let paste2 = global_scope.paste_macro(2, true);
        let prefix = global_scope.get_alias("WHEN_DONE", true);

        let count_var = global_scope
            .get_mut_scope(scope_id)
            .new_var_expr("count", false, None);
        let select_var = global_scope
            .get_mut_scope(scope_id)
            .new_var_expr("select", false, None);
        let discard_var = global_scope
            .get_mut_scope(scope_id)
            .new_var_expr("discard", false, None);
        let on_true_var = global_scope
            .get_mut_scope(scope_id)
            .new_var_expr("a", false, None);
        let on_false_var = global_scope
            .get_mut_scope(scope_id)
            .new_var_expr("b", false, None);

        for (suffix, out_var) in &[
            (self.is_done_true(global_scope), on_true_var),
            (self.is_done_false(global_scope), on_false_var),
        ] {
            global_scope.new_macro(
                &format!("{prefix}_SELECT_{suffix}"),
                out_var.1,
                vec![on_true_var.0, on_false_var.0],
                scope_id,
            );
        }

        let select_call_expr = global_scope
            .get_mut_scope(scope_id)
            .new_expr(ExprContent::List(vec![on_true_var.1, on_false_var.1]), None);
        let paste_text = global_scope
            .get_mut_scope(scope_id)
            .new_expr(ExprContent::Text(format!("{prefix}_SELECT_")), None);
        let select_paste_expr = global_scope.get_mut_scope(scope_id).new_expr(
            ExprContent::List(vec![paste_text, select_var.1]),
            Some((paste2, Some(select_call_expr))),
        );

        let select_macro = global_scope.new_macro(
            &format!("{prefix}_SELECT"),
            select_paste_expr,
            vec![select_var.0, discard_var.0, on_true_var.0, on_false_var.0],
            scope_id,
        );

        let bundle_var = global_scope
            .get_mut_scope(scope_id)
            .new_var_expr("bundle", false, None);
        let expand_expr = global_scope.get_mut_scope(scope_id).new_expr(
            ExprContent::List(vec![bundle_var.1, on_true_var.1, on_false_var.1]),
            Some((select_macro, None)),
        );
        let expand_macro = global_scope.new_macro(
            &format!("{prefix}_SELECT_EXPAND"),
            expand_expr,
            vec![bundle_var.0, on_true_var.0, on_false_var.0],
            scope_id,
        );

        for (suffix, value) in [
            ("0", self.is_done_false(global_scope)),
            ("1", self.is_done_false(global_scope)),
            (
                &self.counter_tail(global_scope),
                self.is_done_true(global_scope),
            ),
        ] {
            global_scope.new_define(format!("{prefix}_FINISHED_{suffix}"), format!("{value},"));
        }

        let finished_paste_text = global_scope
            .get_mut_scope(scope_id)
            .new_expr(ExprContent::Text(format!("{prefix}_FINISHED_")), None);
        let finished_expr = global_scope.get_mut_scope(scope_id).new_expr(
            ExprContent::List(vec![finished_paste_text, count_var.1]),
            Some((paste2, None)),
        );
        let finished_macro = global_scope.new_macro(
            &format!("{prefix}_FINISHED"),
            finished_expr,
            vec![count_var.0],
            scope_id,
        );

        let finished_call_expr = global_scope
            .get_mut_scope(scope_id)
            .new_expr(ExprContent::Var(count_var.0), Some((finished_macro, None)));
        let expr = global_scope.get_mut_scope(scope_id).new_expr(
            ExprContent::List(vec![finished_call_expr, on_true_var.1, on_false_var.1]),
            Some((expand_macro, None)),
        );
        self.when_done = Some(global_scope.new_macro(
            &prefix,
            expr,
            vec![count_var.0, on_true_var.0, on_false_var.0],
            scope_id,
        ));
        self.when_done.unwrap()
    }

    fn counter_tail(&mut self, global_scope: &mut GlobalScope) -> String {
        if let Some(counter_tail) = &self.counter_tail {
            return counter_tail.clone();
        }

        self.counter_tail = Some(global_scope.get_alias("TAIL", false));
        global_scope.new_reserved(self.counter_tail.clone().unwrap());
        self.counter_tail.clone().unwrap()
    }

    fn is_done_true(&mut self, global_scope: &mut GlobalScope) -> String {
        if let Some(is_done_true) = &self.is_done_true {
            return is_done_true.clone();
        }

        self.is_done_true = Some(global_scope.get_alias("TRUE", false));
        global_scope.new_reserved(self.is_done_true.clone().unwrap());
        self.is_done_true.clone().unwrap()
    }

    fn is_done_false(&mut self, global_scope: &mut GlobalScope) -> String {
        if let Some(is_done_false) = &self.is_done_false {
            return is_done_false.clone();
        }

        self.is_done_false = Some(global_scope.get_alias("FALSE", false));
        global_scope.new_reserved(self.is_done_false.clone().unwrap());
        self.is_done_false.clone().unwrap()
    }
}

impl Default for RecursionMacros {
    fn default() -> Self {
        Self::new()
    }
}
