use dto::NTupleOutput;
use parser::parse_gpt_to_features;
use test_case_generator::generate_test_cases_for_multiple_features;

pub mod dto;
mod interval;
mod parser;
mod test_case_generator;
pub mod test_value_generator;
mod util;

pub fn generate_tests_for_gpt_input(input: &str) -> Vec<NTupleOutput> {
    // TODO: Use thiserror and clean up these unwraps
    let (_, features) = parse_gpt_to_features(input).unwrap();
    let test_cases = generate_test_cases_for_multiple_features(&features).unwrap();

    test_cases
}
