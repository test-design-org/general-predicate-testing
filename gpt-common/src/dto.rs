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

impl Intersectable for BoolDTO {
    fn intersects_with(&self, other: &Self) -> bool {
        self.bool_val == other.bool_val
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        if !self.intersects_with(other) {
            return None;
        }

        Some(*self)
    }
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

impl Intersectable for IntervalDTO {
    fn intersects_with(&self, other: &Self) -> bool {
        self.interval.intersects_with(&other.interval)
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        let interval = self.interval.intersect(&other.interval)?;
        Some(Self {
            expression: self.expression,
            interval,
            precision: self.precision,
            is_constant: self.is_constant,
        })
    }
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum Input {
    MissingVariable,
    Bool(BoolDTO),
    Interval(IntervalDTO),
}

impl Intersectable for Input {
    fn intersects_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MissingVariable, _) | (_, Self::MissingVariable) => true,
            (Self::Bool(this), Self::Bool(that)) => this.intersects_with(that),
            (Self::Interval(this), Self::Interval(that)) => this.intersects_with(that),
            (_, _) => false,
        }
    }

    fn intersect(&self, _other: &Self) -> Option<Self> {
        todo!()
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct NTuple {
    pub inputs: Vec<Input>,
}

impl Intersectable for NTuple {
    fn intersects_with(&self, other: &Self) -> bool {
        self.inputs.len() == other.inputs.len()
            && zip(&self.inputs, &other.inputs).all(|(a, b)| a.intersects_with(b))
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        if !self.intersects_with(other) {
            return None;
        }

        let intersected_inputs: Vec<Input> = zip(&self.inputs, &other.inputs)
                .map(|(a, b)| a.intersect(b).expect("When intersecting an NTuple, we checked that each input should intersect, so they should intersect"))
                .collect();

        Some(Self {
            inputs: intersected_inputs,
        })
    }
}
