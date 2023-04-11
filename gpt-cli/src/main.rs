#![warn(
    clippy::all,
    // clippy::restriction,
    // clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use gpt_common::dto::NTupleSingleInterval;
use gpt_common::graph_reduction::{create_graph, MONKE::run_MONKE};
use gpt_common::test_case_generator::generate_test_cases_for_multiple_features;
use gpt_common::{generate_tests_for_gpt_input, parser};

pub fn main() {
    let input1 = r#"
        var VIP: bool
        var price: num
        var second_hand_price: num

        if(VIP == true &&  price < 50) {
            if(second_hand_price == 2)
        }
        if(VIP == false &&  price >= 50)
        if(VIP == true &&  price >= 50)
        if(price > 30 && second_hand_price > 60)
    "#;

    let input2 = r#"
    [
        var heat: int
        var is_contaminated: bool
        var copper: num

        if(heat in [2600,2650] && is_contaminated == false && copper == 8.8)
    ]
    [
        var is_copper_melted: bool
        var tin: num
        var is_contaminated: bool
        if(is_contaminated == false && is_copper_melted == true && tin == 2.2)
    ]
    "#;

    let test_cases = match generate_tests_for_gpt_input(input2) {
        Ok(test_cases) => test_cases,
        Err(e) => panic!("Error: {}", e),
    };

    println!("{:#?}", test_cases);
    println!("Number of test cases: {}", test_cases.len());

    let ntuple_graph = create_graph(&test_cases);
    let monked_graph = run_MONKE(&ntuple_graph);
    let monked_test_cases = monked_graph
        .node_weights()
        .cloned()
        .collect::<Vec<NTupleSingleInterval>>();

    println!("After running MONKE:");
    println!("{:#?}", monked_test_cases);
    println!("Number of test cases: {}", monked_test_cases.len());
}
