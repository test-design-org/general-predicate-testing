use gpt_common::{dto::NTuple, generate_tests_for_gpt_input};
use yew::prelude::*;

use crate::{
    components::{test_case_table::TestCaseTable, usage_guide::UsageGuide},
    text_input::TextInput,
};

#[function_component(ManualTester)]
pub fn manual_tester() -> Html {
    let is_loading = use_state(|| false);
    let is_help_hidden = use_state(|| true);
    let input = use_state(|| {
        r#"
    [
        var heat: int
        var is_contaminated: bool
        var copper: num

        if(heat in [2600,2650] && is_contaminated = false && copper = 8.8)
    ]
    [
        var is_copper_melted: bool
        var tin: num
        var is_contaminated: bool

        if(is_contaminated = false && is_copper_melted = true && tin = 2.2)
    ]
    "#
        .to_owned()
    });
    let generated_state = use_state(|| Some(Vec::<NTuple>::new()));

    let toggle_button_onclick = {
        let is_loading = is_loading.clone();
        let input = input.clone();
        let generated_state = generated_state.clone();
        Callback::from(move |_| {
            is_loading.set(true);
            generated_state.set(None);

            let test_cases = generate_tests_for_gpt_input(&input);

            generated_state.set(Some(test_cases));
            is_loading.set(false);
        })
    };

    let toggle_help_hidden = {
        let is_help_hidden = is_help_hidden.clone();
        Callback::from(move |_| {
            is_help_hidden.set(!*is_help_hidden);
        })
    };

    let on_input_change = {
        let input = input.clone();
        Callback::<String, ()>::from(move |s: String| {
            input.set(s);
        })
    };

    html! {
        <>
      <div class="container">
      <div class="leftInput">
        <h2>{"General predicate test description"}</h2>

        <TextInput value={(*input).clone()} on_change={on_input_change} />

        <h2>{"Notebook"}</h2>

        <textarea rows={10}>
          {"You can use this textarea for notes. It won't have an effect on the
        generated test cases. You can also paste the requirements here, so
        you don't have to switch to other tabs."}
        </textarea>

        <div class="buttons">
          <button onclick={toggle_button_onclick} disabled={*is_loading}>
            {"Generate Tests"}
          </button>

          <button
            onclick={toggle_help_hidden}
            class="toggleButton"
          >
            {if *is_help_hidden { "Open user guide" } else { "Close user guide" }}
          </button>
        </div>
      </div>

      <div class="rightOutput">
        if !*is_loading {
        if let Some(state) = &*generated_state {
          <TestCaseTable
            // variables={state.variables}
            // graph={state.graph}
            test_cases={state.clone()}
          />
        }}
      </div>
    </div>
    <UsageGuide
      class={"usageContainer ".to_owned() + if *is_help_hidden { "hidden" } else { "" }}
    />
    </>
      }
}