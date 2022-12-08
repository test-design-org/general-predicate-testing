use crate::{dto::Expression, interval::MultiInterval};

use super::ast::Type;

#[derive(PartialEq, Clone, Copy)]
pub struct Variable<'a> {
    pub var_name: &'a str,
    pub var_type: Type,
}

#[derive(Clone)]
pub struct BoolCondition<'a> {
    pub var_name: &'a str,
    pub should_equal_to: bool,
}

#[derive(Clone)]
pub struct IntervalCondition<'a> {
    pub var_name: &'a str,
    pub expression: Expression,
    pub interval: MultiInterval,
}

#[derive(Clone)]
pub enum Condition<'a> {
    Bool(BoolCondition<'a>),
    Interval(IntervalCondition<'a>),
}

impl Condition<'_> {
    pub const fn get_variable(&self) -> &str {
        match self {
            Self::Bool(cond) => cond.var_name,
            Self::Interval(cond) => cond.var_name,
        }
    }
}

pub type Predicate<'a> = Vec<Condition<'a>>;

pub struct Feature<'a> {
    pub variables: Vec<Variable<'a>>,
    pub predicates: Vec<Predicate<'a>>,
}
