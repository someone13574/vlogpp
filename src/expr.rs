use std::fmt::{self, Display};
use std::iter;

use crate::global_scope::GlobalScope;
use crate::local_scope::LocalScope;
use crate::r#macro::MacroID;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct VarID(pub usize);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ExprID(pub usize);

pub struct Var {
    pub id: VarID,
    pub name: String,
}

impl Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug)]
pub enum ExprContent {
    Var(VarID),
    Text(String),
    List(Vec<ExprID>),
    Concat(Vec<VarID>),
}

#[derive(Debug)]
pub struct Expr {
    pub id: ExprID,
    pub wrapper: Option<MacroID>,
    pub content: ExprContent,
}

impl Expr {
    pub fn input_vars(&self, local_scope: &LocalScope) -> impl Iterator<Item = VarID> {
        match &self.content {
            ExprContent::Var(var_id) => EitherIter::A(iter::once(*var_id)),
            ExprContent::Text(_) => EitherIter::B(iter::empty::<VarID>()),
            ExprContent::List(expr_ids) => {
                EitherIter::C(expr_ids.iter().flat_map(|expr_id| {
                    local_scope
                        .get_expr(*expr_id)
                        .input_vars(local_scope)
                        .collect::<Vec<_>>()
                        .into_iter()
                }))
            }
            ExprContent::Concat(var_ids) => EitherIter::D(var_ids.iter().copied()),
        }
    }

    pub fn emit(&self, global_scope: &GlobalScope, local_scope: &LocalScope) -> String {
        let content = match &self.content {
            ExprContent::Var(var_id) => {
                format!("{}", local_scope.get_var(*var_id))
            }
            ExprContent::Text(text) => text.to_string(),
            ExprContent::List(expr_ids) => {
                expr_ids
                    .iter()
                    .map(|expr_id| {
                        local_scope
                            .get_expr(*expr_id)
                            .emit(global_scope, local_scope)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            ExprContent::Concat(var_ids) => {
                var_ids
                    .iter()
                    .map(|var_id| format!("{}", local_scope.get_var(*var_id)))
                    .collect::<Vec<_>>()
                    .join("##")
            }
        };

        if let Some(wrapper) = &self.wrapper {
            format!(
                "{}({content})",
                global_scope.macros.get(wrapper).unwrap().name
            )
        } else {
            content
        }
    }
}

pub enum EitherIter<AIterType, BIterType, CIterType, DIterType> {
    A(AIterType),
    B(BIterType),
    C(CIterType),
    D(DIterType),
}

impl<AIterType, BIterType, CIterType, DIterType> Iterator
    for EitherIter<AIterType, BIterType, CIterType, DIterType>
where
    AIterType: Iterator,
    BIterType: Iterator<Item = AIterType::Item>,
    CIterType: Iterator<Item = AIterType::Item>,
    DIterType: Iterator<Item = AIterType::Item>,
{
    type Item = AIterType::Item;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self {
            EitherIter::A(it) => it.next(),
            EitherIter::B(it) => it.next(),
            EitherIter::C(it) => it.next(),
            EitherIter::D(it) => it.next(),
        }
    }
}
