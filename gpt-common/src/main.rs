#![warn(
    clippy::all,
    // clippy::restriction,
    // clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use crate::test_case_generator::generate_test_cases_for_multiple_features;

pub mod dto;
pub mod interval;
pub mod parser;
mod test_case_generator;
mod util;

pub fn main() {
    let input1 = r#"
    [
        var VIP: bool
        var price: num
        var second_hand_price: num

        if(VIP = true &&  price <50) {
            if(second_hand_price = 2)
        }
        if(VIP = false &&  price >=50)
        if(VIP = true &&  price >=50)
        if(price >30 && second_hand_price >60)
    ]
    "#;

    let input2 = r#"
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
    "#;

    let (_, features) = parser::parse_gpt_to_features(input2).unwrap();
    let test_cases = generate_test_cases_for_multiple_features(&features).unwrap();

    println!("{:#?}", test_cases);
    println!("Number of test cases: {}", test_cases.len());
}
