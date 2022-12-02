use dto::Input;

pub mod dto;
pub mod interval;
pub mod parser;
mod test_case_generator;
mod util;

pub fn main() {
    let input = r#"
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
    "#
    .trim();
    let (_, ntuples) = parser::parse_gpt_to_ntuple(input).unwrap();
    let inputs: Vec<Input> = ntuples.into_iter().flat_map(|x| x.inputs).collect();
    let test_cases = test_case_generator::generate_test_cases(&inputs);

    println!("{:?}", test_cases);
}
