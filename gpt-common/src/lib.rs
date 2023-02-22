use dto::NTupleOutput;
use nom::error::convert_error;
use nom::Err;
use parser::parse_gpt_to_features;
use prelude::{GPTError, Result};
use test_case_generator::generate_test_cases_for_multiple_features;

pub mod dto;
pub mod graph_reduction;
mod interval;
mod parser;
pub mod prelude;
mod test_case_generator;
pub mod test_value_generator;
mod util;

pub fn generate_tests_for_gpt_input(input: &str) -> Result<Vec<NTupleOutput>> {
    let (_, features) = parse_gpt_to_features(input).map_err(|error| match error {
        Err::Error(err) | Err::Failure(err) => GPTError::ParseError(convert_error(input, err)),
        Err::Incomplete(err) => GPTError::UnknownParseError(format!("{:?}", err)),
    })?;
    let test_cases = generate_test_cases_for_multiple_features(&features)
        .map_err(|err| GPTError::IntervalError(format!("{:?}", err)))?;

    Ok(test_cases)
}
