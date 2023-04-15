use crate::{dto::Output, interval::Interval};

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

// TODO: There should be a value which returns the whole test case table
pub fn generate_test_value(output: &Output<Interval>, show_interval_values: bool) -> String {
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
                format!(
                    "{:?}",
                    test_value_for_interval(interval)
                        .expect("NTupleSingleInterval should not be empty, it was checked before")
                )
            }
        }
    }
}
