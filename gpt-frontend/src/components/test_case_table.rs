use std::collections::HashSet;

use gpt_common::{
    dto::{NTupleOutput, NTupleSingleInterval, Output},
    interval::{Interval, MultiInterval},
    test_value_generator::generate_test_value,
};
use yew::prelude::*;

/// This takes for example the variable columns of (x, y, z) and the map of {y: 10, x: 30} into [30, 10, *]
fn sort_outputs_into_varible_columns(
    variable_order: &[String],
    ntuple: NTupleSingleInterval,
) -> Vec<Option<Output<Interval>>> {
    variable_order
        .iter()
        .map(|var_name| ntuple.get(&*var_name.clone()).cloned())
        .collect()
}

fn create_test_case_table(
    ntuples: &[NTupleSingleInterval],
) -> (Vec<String>, Vec<Vec<Option<Output<Interval>>>>) {
    let mut variables: Vec<String> =
        HashSet::<String>::from_iter(ntuples.iter().flat_map(|ntuple| ntuple.keys()).cloned())
            .into_iter()
            .collect();

    variables.sort();

    let output = ntuples
        .iter()
        .cloned()
        .map(|ntuple| sort_outputs_into_varible_columns(&variables, ntuple))
        .collect();

    (variables, output)
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    // variables: Vec<String>,
    // graph: String,
    pub test_cases: Vec<NTupleSingleInterval>,
}

#[function_component(TestCaseTable)]
pub fn test_case_table(props: &Props) -> Html {
    let show_interval_values = use_state(|| false);
    let test_case_table_data = {
        let props = props.clone();
        use_memo(
            |test_cases| create_test_case_table(&test_cases.clone()),
            props.test_cases,
        )
    };

    html! {
        <div class="testCaseTable">
      <h2>{"Generated test cases"}</h2>

      <label for="showIntervalValues">
        <input
          type="checkbox"
          checked={*show_interval_values}
          onchange={let show_interval_values = show_interval_values.clone(); move |_| show_interval_values.set(!*show_interval_values)}
          name="showIntervalValues"
          id="showIntervalValues"
        />
        {"Show interval values"}
      </label>

      <br />

      <span>
        <code>{"*"}</code>{" can be any value you like"}
      </span>

      <table class="testValueTable">
        <thead>
          <tr>
            <th></th>
            {(*test_case_table_data.0).iter().map(|var_name|
              html! { <th>{var_name}</th> }
            ).collect::<Html>()}
          </tr>
        </thead>
        <tbody>
        { (*test_case_table_data.1).iter().enumerate().map(|(index, outputs)|
            html!{
              <tr>
                <td>{index + 1}</td>
                {(*outputs).iter().map(|output| html! {
                  <td>
                    {match output {
                      None => "*".to_owned(),
                      Some(output) => generate_test_value(output, *show_interval_values),
                    }}
                  </td>
                }).collect::<Html>()}
              </tr>
            }).collect::<Html>()}
        </tbody>
      </table>
    </div>
    }
}
