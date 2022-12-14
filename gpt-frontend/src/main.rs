mod text_input;

use gpt_common::test_value_generator::generate_test_value;
use gpt_common::{dto::NTuple, generate_tests_for_gpt_input};
use log::info;
// use wasm_bindgen::JsValue;
use crate::text_input::TextInput;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
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
    let test_cases = use_state(|| Vec::<NTuple>::new());
    let onclick = {
        let input = input.clone();
        let test_cases = test_cases.clone();
        move |_| {
            info!("{:?}", input);
            let result = generate_tests_for_gpt_input(&input);
            info!("{:?}", result);
            test_cases.set(result);
        }
    };

    let on_change = {
        let input = input.clone();
        Callback::<String, ()>::from(move |s: String| {
            input.set(s);
        })
    };

    html! {
        <div>
            <button {onclick}>{ "Generate test cases" }</button>
            <TextInput value={(*input).clone()} {on_change} />
            <table>
                { (*test_cases).clone().into_iter().map(|n_tuple| {
                    html!{
                        <tr>
                   {n_tuple.inputs.into_iter().map(|input| html!{ <td> { generate_test_value(&input) } </td>}).collect::<Html>()}
                    </tr>
                    }
                }).collect::<Html>() }
            </table>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
