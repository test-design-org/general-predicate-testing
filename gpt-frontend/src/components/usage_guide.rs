use yew::prelude::*;

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct Props {
    pub class: String,
}

#[function_component(UsageGuide)]
pub fn usage_guide(props: &Props) -> Html {
    html! {
          <div class={&props.class}>
      <h2>{"Usage guide"}</h2>
      <div>
        <p>
          {"The first (non-comment, non-empty) line should be the variable list
          separated by semicolons "}<b>{";"}</b>
          <br />
          {"You can put whitespaces between the expressions (before or after
        semicolons, beginning or end of a line). Whitespaces in the middle of
        expressions are not allowed."}
        </p>
        <p>
          {"Floating point numbers use a dot "}<b>{"."}</b>
        </p>
        <p>
          {"You can leave lines empty, or create a commented line beginning with "}
          <code>{"//"}</code>
        </p>
        <p>
          {"The variables have the following structure:"}
          <ul>
            <li>
              <code>{"variableName(bool)"}</code>
              <br />
              {"variable name with Boolean type"}
            </li>
            <li>
              <code>{"variableName(int)"}</code>
              <br />
              {"variable name with Integer type"}
            </li>
            <li>
              <code>{"variableName(num)"}</code>
              <br />
              {"variable name with Number type, where the default precision is 0.01"}
            </li>
            <li>
              <code>{"variableName(num,0.02)"}</code>
              <br />
              {"variable name wuth Number type, where the precision is given, i.e., 0.02"}
            </li>
          </ul>
        </p>
        <p>
          {"Next, the logical conditions are listed line by line separated with
          semicolons. The semicolons represents semantically AND operators."}
          <br />
          <br />
          {"The atomic conditions can be:"}
          <ul>
            <li>
              <code>{"*"}</code>
              <br />
              {"if there is no constraint on this variable"}
            </li>
            <li>
              <code>{"true"}</code>{" or "}<code>{"false"}</code>
              <br />
              {"if the variable is Boolean"}
            </li>
            <li>
              {"Logical operators followed by a Number, e.g."}
              <br />
              <code>{"<30"}</code>{" or "}<code>{"!=12.62"}</code>
              <br />
              {"The possible operators: "}<code>{"< <= > >= = !="}</code>
            </li>
            <li>
              {"Intervals, e.g. "}<br />
              <code>{"[-2,10.3]"}</code>{" or "}<code>{"[4,10)"}</code>{" or "}
              <code>{"(-2.2,10.3)"}</code>
              <br />
            </li>
          </ul>
          {"You can make an atomic condition a constant by prefixing it with a"}
          <code>{"$"}</code>{", like: "}<code>{"$>=0"}</code>{". "}<br />
          {"You can use constants, where a condition will always be true according
        to the requirements. For example, an age can never be a negative number,
        we can set it to "}<code>{"$>=0"}</code>{". "}<br />
          <br />
          {"Example: Let the condition be the following:"}
          <br />
          <code>
            {"IF isVIP = true AND price > 100 AND price <= 199.99 AND discount > 20"}
          </code>
          <br />
          {"Then the appropriate input is: "}<br />
          <code>{"true; (100,199.99]; >20"}</code>
        </p>
        <p>
          {"Another example with several conditions:"}
          <pre>
            <code>
              {"// Vacation example from https://exercises.test-design.org/paid-vacation-days/"}
              <br />
              {"age(int); service(int)"}
              <br />
              {"<18; *"}
              <br />
              {"[18,45); <15"}
              <br />
              {"[18,45); [15,30)"}
              <br />
              {"[18,60); >=30"}
              <br />
              {"[45,60); <30"}
              <br />
              {">=60; <30"}
              <br />
              {">=60; >=30"}
              <br />
            </code>
          </pre>
        </p>
      </div>
    </div>
      }
}
