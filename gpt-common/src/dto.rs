use std::iter::zip;

use crate::interval::{Intersectable, Interval};

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

#[derive(PartialEq, Clone, Debug, Copy)]
pub struct IntervalDTO {
    pub expression: Expression,
    pub interval: Interval,
    pub precision: f32,
    pub is_constant: bool,
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum Input {
    MissingVariable,
    Bool(BoolDTO),
    Interval(IntervalDTO),
}

#[derive(PartialEq, Clone, Debug)]
pub struct NTupleInput {
    pub inputs: Vec<Input>,
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum Output {
    MissingVariable,
    Bool(bool),
    Interval(Interval),
}

impl Intersectable for Output {
    fn intersects_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MissingVariable, _) | (_, Self::MissingVariable) => true,
            (Self::Bool(this), Self::Bool(that)) => this == that,
            (Self::Interval(this), Self::Interval(that)) => this.intersects_with(that),
            (_, _) => false,
        }
    }

    fn intersect(&self, _other: &Self) -> Option<Self> {
        todo!()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct NTupleOutput {
    pub outputs: Vec<Output>,
}

impl Intersectable for NTupleOutput {
    fn intersects_with(&self, other: &Self) -> bool {
        self.outputs.len() == other.outputs.len()
            && zip(&self.outputs, &other.outputs).all(|(a, b)| a.intersects_with(b))
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        if !self.intersects_with(other) {
            return None;
        }

        let intersected_inputs: Vec<Output> = zip(&self.outputs, &other.outputs)
                .map(|(a, b)| a.intersect(b).expect("When intersecting an NTuple, we checked that each input should intersect, so they should intersect"))
                .collect();

        Some(Self {
            outputs: intersected_inputs,
        })
    }
}
