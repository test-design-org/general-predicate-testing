#![warn(
    clippy::all,
    // clippy::restriction,
    // clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
)]
#![allow(
    illegal_floating_point_literal_pattern,
    clippy::let_and_return,
    clippy::suboptimal_flops,
    clippy::option_if_let_else
)]
#![allow(
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use dto::NTupleSingleInterval;
use ir::Feature;
use nom::{error::convert_error, Err};
use parser::parse_gpt_to_features;
use prelude::{GPTError, Result};
use test_case_generator::generate_test_cases_for_multiple_features;

use crate::parser::parse_gpt_to_ir;

pub mod bva;
pub mod dto;
pub mod graph_reduction;
pub mod interval;
mod ir;
pub mod parser;
pub mod prelude;
pub mod test_case_generator;
pub mod test_value_generator;
mod util;

pub fn and_reduce_gpt_input(input: &str) -> Result<Vec<Feature>> {
    let (_, ir) = parse_gpt_to_ir(input).map_err(|error| match error {
        Err::Error(err) | Err::Failure(err) => GPTError::ParseError(convert_error(input, err)),
        Err::Incomplete(err) => GPTError::UnknownParseError(format!("{err:?}")),
    })?;

    Ok(ir)
}

pub fn generate_tests_for_gpt_input(input: &str) -> Result<Vec<NTupleSingleInterval>> {
    let (_, features) = parse_gpt_to_features(input).map_err(|error| match error {
        Err::Error(err) | Err::Failure(err) => GPTError::ParseError(convert_error(input, err)),
        Err::Incomplete(err) => GPTError::UnknownParseError(format!("{err:?}")),
    })?;
    log::warn!("Inputs: {:#?}", features);
    let test_cases = generate_test_cases_for_multiple_features(&features)
        .map_err(|err| GPTError::IntervalError(format!("{err:?}")))?;

    Ok(test_cases)
}
