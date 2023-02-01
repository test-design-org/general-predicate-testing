use crate::dto::Output;

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
                // TODO: if this is an open interval then this should be one step above. Although GPT works with closed intervals so dunno
                // Also, if the lo / high is -Inf or Inf we should still return a concrete value
                format!("{:?}", interval.lo)
            }
        }
    }
}
