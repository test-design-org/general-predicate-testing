use yew::prelude::*;

#[derive(PartialEq, Eq, Properties)]
pub struct Props {
    pub error_text: String,
}

#[function_component(ErrorDisplay)]
pub fn error_display(props: &Props) -> Html {
    html! {
        <div>
            <h2>{ "Error" }</h2>
            <pre>{ &props.error_text }</pre>
        </div>
    }
}
