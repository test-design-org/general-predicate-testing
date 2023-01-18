use gpt_common::{dto::NTuple, test_value_generator::generate_test_value};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    // variables: Vec<String>,
    // graph: String,
    pub test_cases: Vec<NTuple>,
}

#[function_component(TestCaseTable)]
pub fn test_case_table(props: &Props) -> Html {
    let show_interval_values = use_state(|| false);

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
        // <thead>
        //   <tr>
        //     <th></th>
        //     {variables.map((x) => (
        //       <th>{x.name}</th>
        //     ))}
        //   </tr>
        // </thead>
        <tbody>
        //   {graph.nodes.map((nNuple, index) => (
        //     <tr>
        //       <td>T{index + 1}</td>
        //       {nNuple.list.map((x) => (
        //         <td>{generateTestValue(x, showIntervalValues)}</td>
        //       ))}
        //     </tr>
        //   ))}
        { (props.test_cases).clone().into_iter().map(|n_tuple| {
            html!{
                <tr>
           {n_tuple.inputs.into_iter().map(|input| html!{ <td> { generate_test_value(&input, *show_interval_values) } </td>}).collect::<Html>()}
            </tr>
            }
        }).collect::<Html>() }
        </tbody>
      </table>
    </div>
    }
}
