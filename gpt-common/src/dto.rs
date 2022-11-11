use std::iter::zip;

use crate::{
    interval::{Intersectable, Interval},
    parser::ast::EqOp,
};

#[derive(Clone, PartialEq)]
pub struct BoolDTO {
    expression: EqOp,
    bool_val: bool,
    is_constant: bool,
}

impl Intersectable for BoolDTO {
    fn intersects_with(&self, other: &BoolDTO) -> bool {
        self.bool_val == other.bool_val
    }

    fn intersect(&self, other: &BoolDTO) -> Option<BoolDTO> {
        if !self.intersects_with(other) {
            None
        } else {
            Some(self.clone())
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
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

#[derive(PartialEq, Clone)]
pub struct IntervalDTO {
    pub expression: Expression,
    pub interval: Interval,
    pub precision: f32,
    pub is_constant: bool,
}

impl Intersectable for IntervalDTO {
    fn intersects_with(&self, other: &IntervalDTO) -> bool {
        self.interval.intersects_with(&other.interval)
    }

    fn intersect(&self, other: &IntervalDTO) -> Option<IntervalDTO> {
        let interval = self.interval.intersect(&other.interval)?;
        Some(IntervalDTO {
            expression: self.expression,
            interval,
            precision: self.precision,
            is_constant: self.is_constant,
        })
    }
}

#[derive(PartialEq, Clone)]
pub enum Input {
    MissingVariable,
    Bool(BoolDTO),
    Interval(IntervalDTO),
}

impl Intersectable for Input {
    fn intersects_with(&self, other: &Input) -> bool {
        match (self, other) {
            (Input::MissingVariable, _) => true,
            (_, Input::MissingVariable) => true,
            (Input::Bool(this), Input::Bool(that)) => this.intersects_with(that),
            (Input::Interval(this), Input::Interval(that)) => this.intersects_with(that),
            (_, _) => false,
        }
    }

    fn intersect(&self, other: &Input) -> Option<Input> {
        todo!()
    }
}

#[derive(PartialEq, Clone)]
pub struct NTuple {
    pub inputs: Vec<Input>,
}

impl Intersectable for NTuple {
    fn intersects_with(&self, other: &NTuple) -> bool {
        self.inputs.len() == other.inputs.len()
            && zip(&self.inputs, &other.inputs).all(|(a, b)| a.intersects_with(b))
    }

    fn intersect(&self, other: &NTuple) -> Option<NTuple> {
        if !self.intersects_with(other) {
            None
        } else {
            let intersected_inputs: Vec<Input> = zip(&self.inputs, &other.inputs)
                .map(|(a, b)| a.intersect(b).expect("When intersecting an NTuple, we checked that each input should intersect, so they should intersect"))
                .collect();

            Some(NTuple {
                inputs: intersected_inputs,
            })
        }
    }
}
