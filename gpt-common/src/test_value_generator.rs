use crate::dto::{BoolDTO, Input, IntervalDTO};

pub fn generate_test_value(input: &Input, show_interval_values: bool) -> String {
    match input {
        Input::MissingVariable => "*".to_owned(),
        Input::Bool(BoolDTO { bool_val, .. }) => {
            if *bool_val == true {
                "true".to_owned()
            } else {
                "false".to_owned()
            }
        }
        Input::Interval(IntervalDTO { interval, .. }) => {
            if show_interval_values {
                format!("{:?}", interval)
            } else {
                // TODO: if this is an open interval then this should be one step above. Although GPT works with closed intervals so dunno
                // Also, if the lo / high is -Inf or Inf we should still return a concrete value
                format!("{:?}", interval.lo)
            }
        }
    }
}
