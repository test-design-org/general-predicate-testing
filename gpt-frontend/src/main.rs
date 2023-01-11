mod components;
mod text_input;

use crate::components::manual_tester::ManualTester;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    html! {
        <div>
            <a
                href="https://test-design.org/practical-exercises/"
                class="backLink"
            >{ "Back to Test Design Exercises" }</a>

            <ManualTester />
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
