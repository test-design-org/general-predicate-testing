use std::fmt::Debug;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::interval::{Intersectable, Interval, MultiInterval};

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum BoolExpression {
    IsTrue,
    IsFalse,
}

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub struct BoolDTO {
    pub expression: BoolExpression,
    pub bool_val: bool,
    pub is_constant: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Expression {
    LessThan,
    LessThanOrEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
    EqualTo,
    NotEqualTo,
    //   BoolTrue,
    //   BoolFalse,
    Interval,
    //   MissingVariable,
}

#[derive(PartialEq, Clone, Debug)]
pub struct IntervalDTO {
    pub expression: Expression,
    pub interval: MultiInterval,
    pub precision: f32,
    pub is_constant: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Input {
    Bool(BoolDTO),
    Interval(IntervalDTO),
}

#[derive(PartialEq, Clone, Debug)]
pub struct NTupleInput {
    pub inputs: HashMap<String, Input>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Output {
    MissingVariable,
    Bool(bool),
    Interval(MultiInterval),
}

impl Intersectable for Output {
    fn intersects_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(this), Self::Bool(that)) => this == that,
            (Self::Interval(this), Self::Interval(that)) => this.intersects_with(that),
            (_, _) => false,
        }
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        match (self, other) {
            (Self::Bool(this), Self::Bool(that)) => Some(Self::Bool(*this)),
            (Self::Interval(this), Self::Interval(that)) => this
                .intersect(that)
                .map(|interval| Self::Interval(interval)),
            (Self::MissingVariable, Self::MissingVariable) => Some(Self::MissingVariable),
            (_, _) => None,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct NTupleOutput {
    pub outputs: HashMap<String, Output>,
}

impl Intersectable for NTupleOutput {
    fn intersects_with(&self, other: &Self) -> bool {
        for (var_name, input) in self.outputs.iter() {
            if let Some(other_input) = other.outputs.get(var_name) {
                if !input.intersects_with(other_input) {
                    return false;
                }
            }
        }

        true
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        if !self.intersects_with(other) {
            return None;
        }

        let var_names_in_both =
            HashSet::<&String>::from_iter(self.outputs.keys().chain(other.outputs.keys()));

        let intersected_outputs: HashMap<String, Output> = var_names_in_both.iter().filter_map(|var_name| {
            let var_name = (*var_name).clone();
            let intersection = match (self.outputs.get(&*var_name), other.outputs.get(&*var_name)) {
                (None, None) => panic!("in NTuple intersection, variable name should be at least in one of the maps, because we use keys from the maps"),
                (Some(x), None) => Some(x.clone()),
                (None, Some(y)) => Some(y.clone()),
                (Some(x), Some(y)) => x.intersect(y),
            }?;

            Some((var_name, intersection))
        })
        .map(|(var_name, input)| (var_name, input))
        .collect::<HashMap<String, Output>>();

        Some(Self {
            outputs: intersected_outputs,
        })
    }
}

impl Display for NTupleOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (var_name, interval) in self.outputs.iter() {
            write!(f, " {var_name}: {:?}", interval)?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl Debug for NTupleOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;

    use pretty_assertions::{assert_eq, assert_ne};
    use rstest::rstest;

    use super::{Input, NTupleInput, NTupleOutput, Output};
    use crate::interval::{test::multiint, Intersectable};

    pub fn create_ntuple_input(inputs: Vec<(&str, Input)>) -> NTupleInput {
        NTupleInput {
            inputs: HashMap::from_iter(
                inputs
                    .into_iter()
                    .map(|(var_name, input)| (var_name.to_owned(), input)),
            ),
        }
    }

    pub fn create_ntuple_output(outputs: Vec<(&str, Output)>) -> NTupleOutput {
        NTupleOutput {
            outputs: HashMap::from_iter(
                outputs
                    .into_iter()
                    .map(|(var_name, output)| (var_name.to_owned(), output)),
            ),
        }
    }

    #[rstest]
    #[case::same(vec![
        ("x", Output::Interval(multiint("[10, 20]"))),
        ("y", Output::Bool(true))
    ],vec![
        ("x", Output::Interval(multiint("[10, 20]"))),
        ("y", Output::Bool(true))
    ])]
    #[case::non_intersectable_different_variables(vec![
        ("x", Output::Interval(multiint("[0, 100]"))),
        ("y", Output::Bool(true))
    ],vec![
        ("x", Output::Interval(multiint("[10, 20]"))),
        ("z", Output::Bool(false))
    ])]
    #[case::empty_left(vec![], vec![
        ("x", Output::Interval(multiint("[10, 20]"))),
        ("z", Output::Bool(false))
    ])]
    #[case::empty_right(vec![
        ("x", Output::Interval(multiint("[0, 100]"))),
        ("y", Output::Bool(true))
    ],vec![])]
    #[case::both_empty(vec![], vec![])]
    fn test_ntuple_intersects_with(
        #[case] left: Vec<(&str, Output)>,
        #[case] right: Vec<(&str, Output)>,
    ) {
        assert!(create_ntuple_output(left).intersects_with(&create_ntuple_output(right)));
    }

    #[rstest]
    #[case::non_intersectable_same_variables(vec![
        ("x", Output::Interval(multiint("[10, 20]"))),
        ("y", Output::Bool(true))
    ],vec![
        ("x", Output::Interval(multiint("[10, 20]"))),
        ("y", Output::Bool(false))
    ])]
    fn test_ntuple_not_intersects_with(
        #[case] left: Vec<(&str, Output)>,
        #[case] right: Vec<(&str, Output)>,
    ) {
        assert!(!create_ntuple_output(left).intersects_with(&create_ntuple_output(right)));
    }
}
