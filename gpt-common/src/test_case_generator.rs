use std::collections::HashMap;

use crate::{
    bva::Bva,
    dto::{
        BoolDTO, BoolExpression, Input, IntervalDTO, NTupleInput, NTupleOutput,
        NTupleSingleInterval, Output,
    },
    interval::{IntervalError, MultiInterval},
    util::UniquesVec,
};

pub fn generate_test_cases_for_multiple_features(
    features: &Vec<Vec<NTupleInput>>,
) -> Result<Vec<NTupleSingleInterval>, IntervalError> {
    let mut res = Vec::new();
    for feature in features {
        let mut test_cases = generate_test_cases_for_feature(feature);
        res.append(&mut test_cases);
    }
    Ok(res)
}

fn generate_test_cases_for_feature(n_tuples: &[NTupleInput]) -> Vec<NTupleSingleInterval> {
    let mut result_test_cases = Vec::new();
    for ntuple in n_tuples {
        let mut test_cases = generate_test_cases_for_inputs(&ntuple.clone());
        result_test_cases.append(&mut test_cases);
    }

    result_test_cases
}

fn generate_test_cases_for_inputs(inputs: &NTupleInput) -> Vec<NTupleSingleInterval> {
    let mut modified_inputs = calc_in_on_inin(inputs);
    modified_inputs.extend(off_out(inputs));

    modified_inputs = modified_inputs.uniques();

    modified_inputs
        .iter()
        .flat_map(ntuple_multi_cartesian_product)
        .collect()
}

/// Creates the cartesian product of all multiintervals in the NTuple.
/// If a multiinterval would have multiple intervals, it creates an NTuple with all the possible single interval combinations.
fn ntuple_multi_cartesian_product(ntuple: &NTupleOutput) -> Vec<NTupleSingleInterval> {
    if ntuple.outputs.iter().any(|(_, output)| match output {
        Output::MissingVariable => false,
        Output::Bool(_) => false,
        Output::Interval(interval) => interval.is_empty(),
    }) {
        return Vec::new();
    }

    let mut res: Vec<NTupleSingleInterval> = Vec::new();

    let first: NTupleSingleInterval = ntuple
        .outputs
        .iter()
        .map(|(var_name, output)| {
            (
                var_name.to_owned(),
                match output {
                    Output::MissingVariable => Output::MissingVariable,
                    Output::Bool(x) => Output::Bool(*x),
                    Output::Interval(x) => Output::Interval(x.intervals[0]),
                },
            )
        })
        .collect();

    res.push(first);

    for (var_name, output) in ntuple.outputs.iter()
    // .sorted_unstable_by_key(|(var_name, _)| var_name.to_owned())
    {
        let current = res.clone();

        match output {
            Output::MissingVariable | Output::Bool(_) => (),
            Output::Interval(interval) => {
                let mut new = Vec::new();
                for interval in interval.intervals.iter().skip(1) {
                    for x in current.iter() {
                        let mut x = x.clone();
                        x.insert(var_name.clone(), Output::Interval(*interval));
                        new.push(x);
                    }
                }
                res.append(&mut new);
            }
        }
    }

    res
}

fn calc_in_on_inin(ntuple: &NTupleInput) -> Vec<NTupleOutput> {
    fn input_to_output(
        ntuple: &NTupleInput,
        f: impl Fn(&MultiInterval, f32) -> MultiInterval,
    ) -> HashMap<String, Output<MultiInterval>> {
        ntuple
            .inputs
            .iter()
            .map(|(var_name, input)| -> (String, Output<MultiInterval>) {
                let output = match input {
                    Input::Bool(BoolDTO { bool_val, .. }) => Output::Bool(*bool_val),
                    Input::Interval(IntervalDTO {
                        is_constant,
                        interval,
                        ..
                    }) if *is_constant => Output::Interval(interval.clone()),
                    Input::Interval(IntervalDTO {
                        interval,
                        precision,
                        ..
                    }) => Output::Interval(f(interval, *precision)),
                };

                (var_name.clone(), output)
            })
            .collect()
    }

    let ins = input_to_output(ntuple, |interval, precision| {
        match interval.calc_in(precision) {
            x if x.is_empty() => interval.on(precision),
            x => x,
        }
    });

    let ons = input_to_output(ntuple, |interval, precision| interval.on(precision));

    let inins = input_to_output(ntuple, |interval, precision| {
        match interval.inin(precision) {
            x if x.is_empty() => interval.on(precision),
            x => x,
        }
    });
    vec![
        NTupleOutput { outputs: ins },
        NTupleOutput { outputs: ons },
        NTupleOutput { outputs: inins },
    ]
}

fn baseline(ntuple: &NTupleInput) -> NTupleOutput {
    let outputs = ntuple
        .inputs
        .clone()
        .into_iter()
        .map(|(var_name, input)| {
            let outputs = match input {
                Input::Bool(BoolDTO { bool_val, .. }) => Output::Bool(bool_val),
                Input::Interval(IntervalDTO {
                    interval,
                    precision,
                    ..
                }) => Output::Interval(interval.calc_in(precision)),
            };

            (var_name, outputs)
        })
        .collect::<HashMap<String, Output<MultiInterval>>>();

    NTupleOutput { outputs }
}

fn off_out(ntuple: &NTupleInput) -> Vec<NTupleOutput> {
    let mut output: Vec<NTupleOutput> = Vec::new();
    let base = baseline(ntuple);

    for (i, input) in ntuple.inputs.iter() {
        match input {
            Input::Bool(BoolDTO { is_constant, .. }) if *is_constant => continue,
            Input::Interval(IntervalDTO { is_constant, .. }) if *is_constant => continue,
            _ => (),
        }

        match input {
            Input::Bool(BoolDTO { expression, .. }) => match expression {
                BoolExpression::IsTrue => {
                    let mut base_bool_true = base.clone();
                    base_bool_true
                        .outputs
                        .insert(i.to_owned(), Output::Bool(false));
                    output.push(base_bool_true);
                }
                BoolExpression::IsFalse => {
                    let mut base_bool_false = base.clone();
                    base_bool_false
                        .outputs
                        .insert(i.to_owned(), Output::Bool(true));
                    output.push(base_bool_false);
                }
            },
            Input::Interval(IntervalDTO {
                interval,
                precision,
                ..
            }) => {
                let mut base_off = base.clone();
                base_off
                    .outputs
                    .insert(i.to_owned(), Output::Interval(interval.off(*precision)));

                let mut base_out = base.clone();
                base_out
                    .outputs
                    .insert(i.to_owned(), Output::Interval(interval.out(*precision)));

                output.push(base_off);
                output.push(base_out);
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;
    use Boundary::Open;

    use super::{generate_test_cases_for_inputs, ntuple_multi_cartesian_product};
    use crate::{
        dto::{
            tests::{create_ntuple_input, create_ntuple_output, create_ntuple_single_interval},
            BoolDTO, BoolExpression, Input, IntervalDTO, NTupleSingleInterval, Output,
        },
        interval::{
            test::{int, multiint},
            Boundary, Interval, MultiInterval,
        },
    };

    #[test]
    fn test_all_possible_combinations() {
        let input = create_ntuple_output(vec![
            ("a", Output::Interval(multiint("[1,1]"))),
            ("b", Output::Bool(true)),
            ("c", Output::Interval(multiint("[2,2] [3,3] [4,4]"))),
            ("d", Output::Interval(multiint("[5,5] [6,6]"))),
            ("e", Output::Interval(multiint("[7,7] [8,8] [9,9]"))),
        ]);

        let expected: Vec<NTupleSingleInterval> = vec![
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[2,2]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[7,7]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[3,3]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[7,7]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[4,4]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[7,7]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[2,2]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[7,7]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[3,3]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[7,7]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[4,4]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[7,7]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[2,2]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[8,8]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[3,3]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[8,8]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[4,4]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[8,8]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[2,2]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[8,8]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[3,3]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[8,8]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[4,4]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[8,8]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[2,2]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[9,9]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[3,3]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[9,9]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[4,4]"))),
                ("d".to_owned(), Output::Interval(int("[5,5]"))),
                ("e".to_owned(), Output::Interval(int("[9,9]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[2,2]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[9,9]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[3,3]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[9,9]"))),
            ]),
            HashMap::from([
                ("a".to_owned(), Output::Interval(int("[1,1]"))),
                ("b".to_owned(), Output::Bool(true)),
                ("c".to_owned(), Output::Interval(int("[4,4]"))),
                ("d".to_owned(), Output::Interval(int("[6,6]"))),
                ("e".to_owned(), Output::Interval(int("[9,9]"))),
            ]),
        ];

        let result = ntuple_multi_cartesian_product(&input);

        // println!("Result:");
        // for r in result.iter() {
        //     print!("[");
        //     for (k, v) in r.iter().sorted_by_key(|(k, _)| k.to_owned()) {
        //         print!(
        //             "{} ",
        //             match v {
        //                 Output::Interval(i) => format!("{:?}", i),
        //                 Output::Bool(b) => b.to_string(),
        //                 Output::MissingVariable => "*".to_owned(),
        //             }
        //         );
        //     }
        //     println!("]");
        // }

        // println!("Expected:");
        // for r in expected.iter() {
        //     print!("[");
        //     for (k, v) in r.iter().sorted_by_key(|(k, _)| k.to_owned()) {
        //         print!(
        //             "{} ",
        //             match v {
        //                 Output::Interval(i) => i.lo.to_string(),
        //                 Output::Bool(b) => b.to_string(),
        //                 Output::MissingVariable => "*".to_owned(),
        //             }
        //         );
        //     }
        //     println!("]");
        // }

        assert_eq!(result.len(), expected.len());
        assert!(result.iter().all(|x| expected.contains(x)));
        assert!(expected.iter().all(|x| result.contains(x)));
    }

    #[test]
    fn test_generate_test_cases_for_inputs() {
        // true;   <50; *
        let inputs = create_ntuple_input(vec![
            (
                "x",
                Input::Bool(BoolDTO {
                    expression: BoolExpression::IsTrue,
                    bool_val: true,
                    is_constant: false,
                }),
            ),
            (
                "y",
                Input::Interval(IntervalDTO {
                    interval: MultiInterval::new(Open, f32::NEG_INFINITY, 50.0, Open).unwrap(),
                    precision: 0.01,
                    is_constant: false,
                }),
            ),
        ]);

        let expected: Vec<NTupleSingleInterval> = vec![
            // in
            create_ntuple_single_interval(vec![
                ("x", Output::Bool(true)),
                (
                    "y",
                    Output::Interval(Interval::new_closed(f32::NEG_INFINITY, 49.99).unwrap()),
                ),
            ]),
            // on
            create_ntuple_single_interval(vec![
                ("x", Output::Bool(true)),
                ("y", Output::Interval(Interval::new_closed_point(49.99))),
            ]),
            // inin
            create_ntuple_single_interval(vec![
                ("x", Output::Bool(true)),
                (
                    "y",
                    Output::Interval(Interval::new_closed(f32::NEG_INFINITY, 49.98).unwrap()),
                ),
            ]),
            // Bool False
            create_ntuple_single_interval(vec![
                ("x", Output::Bool(false)),
                (
                    "y",
                    Output::Interval(Interval::new_closed(f32::NEG_INFINITY, 49.99).unwrap()),
                ),
            ]),
            // Out
            create_ntuple_single_interval(vec![
                ("x", Output::Bool(true)),
                (
                    "y",
                    Output::Interval(Interval::new_closed(50.01, f32::INFINITY).unwrap()),
                ),
            ]),
            // Off
            create_ntuple_single_interval(vec![
                ("x", Output::Bool(true)),
                ("y", Output::Interval(Interval::new_closed_point(50.0))),
            ]),
        ];

        let result = generate_test_cases_for_inputs(&inputs);

        // println!("Expected: {:#?}", expected);
        // println!("Result: {:#?}", result);

        assert_eq!(result.len(), expected.len());
        assert!(result.iter().all(|x| expected.contains(x)));
        assert!(expected.iter().all(|x| result.contains(x)));
    }
}
