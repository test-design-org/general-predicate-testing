use crate::{
    dto::Output,
    interval::{Interval, MultiInterval},
};

fn test_value_for_interval(interval: &Interval) -> Option<f32> {
    if interval.is_empty() {
        None
    } else {
        Some(match (interval.lo, interval.hi) {
            (f32::NEG_INFINITY, f32::INFINITY) => 0.0,
            (f32::NEG_INFINITY, x) => x,
            (x, f32::INFINITY) => x,
            (x, _) => x,
        })
    }
}

fn test_values_for_multiinterval(interval: &MultiInterval) -> Vec<f32> {
    interval
        .intervals
        .iter()
        .flat_map(|int| test_value_for_interval(int))
        .collect()
}

// TODO: There should be a value which returns the whole test case table
//       This is an issue, because for MultiIntervals there could be multiple test values, which would mean multiple test cases
pub fn generate_test_value(output: &Output, show_interval_values: bool) -> String {
    match output {
        Output::MissingVariable => "*".to_owned(),
        Output::Bool(bool_val) => {
            if *bool_val == true {
                "true".to_owned()
            } else {
                "false".to_owned()
            }
        }
        Output::Interval(interval) => {
            if show_interval_values {
                format!("{:?}", interval)
            } else {
                format!("{:?}", test_values_for_multiinterval(interval))
            }
        }
    }
}
