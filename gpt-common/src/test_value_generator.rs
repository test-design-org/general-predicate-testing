use crate::dto::{BoolDTO, Input, IntervalDTO};

pub fn generate_test_value(input: &Input) -> String {
    match input {
        Input::MissingVariable => "*".to_owned(),
        Input::Bool(BoolDTO { bool_val, .. }) => {
            if *bool_val == true {
                "true".to_owned()
            } else {
                "false".to_owned()
            }
        }
        Input::Interval(IntervalDTO { interval, .. }) => format!("{:?}", interval),
    }
}
