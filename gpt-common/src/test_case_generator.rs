use std::collections::HashMap;

use crate::bva::Bva;
use crate::dto::{BoolDTO, BoolExpression, NTupleOutput, Output};
use crate::interval::MultiInterval;
use crate::util::UniquesVec;

use crate::{
    dto::{Input, IntervalDTO, NTupleInput},
    interval::IntervalError,
};

pub fn generate_test_cases_for_multiple_features(
    features: &Vec<Vec<NTupleInput>>,
) -> Result<Vec<NTupleOutput>, IntervalError> {
    let mut res = Vec::new();
    for feature in features {
        let mut test_cases = generate_test_cases_for_feature(feature);
        res.append(&mut test_cases);
    }
    Ok(res)
}

fn generate_test_cases_for_feature(n_tuples: &[NTupleInput]) -> Vec<NTupleOutput> {
    let mut result_test_cases = Vec::new();
    for ntuple in n_tuples {
        let mut test_cases = generate_test_cases_for_inputs(&ntuple.clone());
        result_test_cases.append(&mut test_cases);
    }

    result_test_cases
}

fn generate_test_cases_for_inputs(inputs: &NTupleInput) -> Vec<NTupleOutput> {
    let mut modified_inputs = calc_in_on_inin(inputs);
    modified_inputs.extend(off_out(inputs));

    modified_inputs = modified_inputs.uniques();

    modified_inputs
}

fn calc_in_on_inin(ntuple: &NTupleInput) -> Vec<NTupleOutput> {
    fn input_to_output(
        ntuple: &NTupleInput,
        f: impl Fn(&MultiInterval, f32) -> MultiInterval,
    ) -> HashMap<String, Output> {
        ntuple
            .inputs
            .iter()
            .map(|(var_name, input)| -> (String, Output) {
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
        .collect::<HashMap<String, Output>>();

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
    use crate::dto::tests::{create_ntuple_input, create_ntuple_output};
    use crate::interval::MultiInterval;
    use crate::{
        dto::{BoolDTO, BoolExpression, Expression, Input, IntervalDTO, Output},
        interval::Boundary,
    };

    use Boundary::{Closed, Open};

    use super::generate_test_cases_for_inputs;

    #[test]
    fn test_generate_test_cases_for_inputs() {
        // true;   <50; *
        let inputs = create_ntuple_input(vec![
            (
                "x".to_owned(),
                Input::Bool(BoolDTO {
                    expression: BoolExpression::IsTrue,
                    bool_val: true,
                    is_constant: false,
                }),
            ),
            (
                "y".to_owned(),
                Input::Interval(IntervalDTO {
                    expression: Expression::LessThan,
                    interval: MultiInterval::new(Open, f32::NEG_INFINITY, 50.0, Open).unwrap(),
                    precision: 0.01,
                    is_constant: false,
                }),
            ),
        ]);

        let expected = vec![
            // in
            create_ntuple_output(vec![
                ("x".to_owned(), Output::Bool(true)),
                (
                    "y".to_owned(),
                    Output::Interval(MultiInterval::new_closed(f32::NEG_INFINITY, 49.99).unwrap()),
                ),
            ]),
            // on
            create_ntuple_output(vec![
                ("x".to_owned(), Output::Bool(true)),
                (
                    "y".to_owned(),
                    Output::Interval(MultiInterval::new_closed_point(49.99)),
                ),
            ]),
            // inin
            create_ntuple_output(vec![
                ("x".to_owned(), Output::Bool(true)),
                (
                    "y".to_owned(),
                    Output::Interval(MultiInterval::new_closed(f32::NEG_INFINITY, 49.98).unwrap()),
                ),
            ]),
            // Bool False
            create_ntuple_output(vec![
                ("x".to_owned(), Output::Bool(false)),
                (
                    "y".to_owned(),
                    Output::Interval(MultiInterval::new_closed(f32::NEG_INFINITY, 49.99).unwrap()),
                ),
            ]),
            // Out
            create_ntuple_output(vec![
                ("x".to_owned(), Output::Bool(true)),
                (
                    "y".to_owned(),
                    Output::Interval(MultiInterval::new_closed(50.01, f32::INFINITY).unwrap()),
                ),
            ]),
            // Off
            create_ntuple_output(vec![
                ("x".to_owned(), Output::Bool(true)),
                (
                    "y".to_owned(),
                    Output::Interval(MultiInterval::new_closed_point(50.0)),
                ),
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
