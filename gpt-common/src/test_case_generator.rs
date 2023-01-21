use crate::dto::{BoolDTO, BoolExpression, Expression, NTupleOutput, Output};
use crate::interval::Interval;
use crate::util::UniquesVec;

use crate::{
    dto::{Input, IntervalDTO, NTupleInput},
    interval::{Boundary, IntervalError},
};

pub fn generate_test_cases_for_multiple_features(
    features: &Vec<Vec<NTupleInput>>,
) -> Result<Vec<NTupleOutput>, IntervalError> {
    let mut res = Vec::new();
    for feature in features {
        let mut test_cases = generate_test_cases_for_feature(feature)?;
        res.append(&mut test_cases);
    }
    Ok(res)
}

fn generate_test_cases_for_feature(
    n_tuples: &[NTupleInput],
) -> Result<Vec<NTupleOutput>, IntervalError> {
    let inputs = n_tuples
        .iter()
        .flat_map(|x| x.inputs.clone())
        .collect::<Vec<Input>>();
    generate_test_cases_for_inputs(&inputs)
}

fn generate_test_cases_for_inputs(inputs: &[Input]) -> Result<Vec<NTupleOutput>, IntervalError> {
    let mut modified_inputs = vec![
        calculate_in_on_patterns1(inputs)?,
        calculate_in_on_patterns2(inputs)?,
    ];
    modified_inputs.extend(off_out(inputs)?);

    let mut n_tuples: Vec<NTupleOutput> = modified_inputs
        .into_iter()
        .map(|outputs| NTupleOutput { outputs })
        .collect();

    n_tuples = n_tuples.uniques();

    Ok(n_tuples)
}

fn calculate_in_on_patterns1(inputs: &[Input]) -> Result<Vec<Output>, IntervalError> {
    inputs
        .iter()
        .map(|input| -> Result<Output, IntervalError> {
            match input {
                Input::MissingVariable => Ok(Output::MissingVariable),
                Input::Bool(BoolDTO { bool_val, .. }) => Ok(Output::Bool(*bool_val)),
                Input::Interval(IntervalDTO {
                    is_constant,
                    interval,
                    ..
                }) if *is_constant => Ok(Output::Interval(*interval)),
                Input::Interval(interval_dto) => match interval_dto.expression {
                    Expression::LessThan | Expression::LessThanOrEqualTo => {
                        in_in(interval_dto, InInVersion::IntervalRight)
                    }

                    Expression::EqualTo => Ok(on(interval_dto, OnVersion::IntervalEqual)),

                    Expression::GreaterThan | Expression::GreaterThanOrEqualTo => {
                        in_in(interval_dto, InInVersion::IntervalLeft)
                    }

                    Expression::NotEqualTo => calc_in(interval_dto, InVersion::IntervalRight),
                    Expression::Interval => Ok(on(interval_dto, OnVersion::IntervalLeft)),
                },
            }
        })
        .collect()
}

fn calculate_in_on_patterns2(inputs: &[Input]) -> Result<Vec<Output>, IntervalError> {
    inputs
        .iter()
        .map(|input| -> Result<Output, IntervalError> {
            match input {
                Input::MissingVariable => Ok(Output::MissingVariable),
                Input::Bool(BoolDTO { bool_val, .. }) => Ok(Output::Bool(*bool_val)),
                Input::Interval(IntervalDTO {
                    is_constant,
                    interval,
                    ..
                }) if *is_constant => Ok(Output::Interval(*interval)),
                Input::Interval(interval_dto) => match interval_dto.expression {
                    Expression::LessThan | Expression::LessThanOrEqualTo => {
                        Ok(on(interval_dto, OnVersion::IntervalRight))
                    }

                    Expression::EqualTo => Ok(on(interval_dto, OnVersion::IntervalEqual)),

                    Expression::GreaterThan | Expression::GreaterThanOrEqualTo => {
                        Ok(on(interval_dto, OnVersion::IntervalLeft))
                    }

                    Expression::NotEqualTo => calc_in(interval_dto, InVersion::IntervalLeft),
                    Expression::Interval => Ok(on(interval_dto, OnVersion::IntervalRight)),
                },
            }
        })
        .collect()
}

fn baseline(inputs: &[Input]) -> Result<Vec<Output>, IntervalError> {
    inputs
        .iter()
        .map(|input| -> Result<Output, IntervalError> {
            match input {
                Input::MissingVariable => Ok(Output::MissingVariable),
                Input::Bool(BoolDTO { bool_val, .. }) => Ok(Output::Bool(*bool_val)),
                Input::Interval(interval_dto) => match interval_dto.expression {
                    Expression::LessThan | Expression::LessThanOrEqualTo => {
                        calc_in(interval_dto, InVersion::IntervalRight)
                    }

                    Expression::GreaterThan
                    | Expression::GreaterThanOrEqualTo
                    | Expression::NotEqualTo => calc_in(interval_dto, InVersion::IntervalLeft),

                    Expression::EqualTo => Ok(on(interval_dto, OnVersion::IntervalEqual)),
                    Expression::Interval => calc_in(interval_dto, InVersion::Interval),
                },
            }
        })
        .collect()
}

fn off_out(inputs: &[Input]) -> Result<Vec<Vec<Output>>, IntervalError> {
    let mut output: Vec<Vec<Output>> = Vec::new();

    for (i, input) in inputs.iter().enumerate() {
        match input {
            Input::MissingVariable => continue,
            Input::Bool(BoolDTO { is_constant, .. }) if *is_constant => continue,
            Input::Interval(IntervalDTO { is_constant, .. }) if *is_constant => continue,
            _ => (),
        }

        let mut based1 = baseline(inputs)?;
        let mut based2 = baseline(inputs)?;
        let mut based3 = baseline(inputs)?;
        let mut based4 = baseline(inputs)?;

        match input {
            Input::MissingVariable => (),
            Input::Bool(BoolDTO { expression, .. }) => match expression {
                BoolExpression::IsTrue => {
                    based1[i] = Output::Bool(false);
                    output.push(based1);
                }
                BoolExpression::IsFalse => {
                    based1[i] = Output::Bool(true);
                    output.push(based1);
                }
            },
            Input::Interval(interval_dto @ IntervalDTO { expression, .. }) => match expression {
                Expression::LessThan | Expression::LessThanOrEqualTo => {
                    based1[i] = out(interval_dto, OutVersion::IntervalRight)?;
                    based2[i] = off(interval_dto, OffVersion::IntervalRight);

                    output.push(based1);
                    output.push(based2);
                }
                Expression::GreaterThan | Expression::GreaterThanOrEqualTo => {
                    based1[i] = out(interval_dto, OutVersion::IntervalLeft)?;
                    based2[i] = off(interval_dto, OffVersion::IntervalLeft);

                    output.push(based1);
                    output.push(based2);
                }
                Expression::EqualTo => {
                    based1[i] = out(interval_dto, OutVersion::Right)?;
                    based2[i] = out(interval_dto, OutVersion::Left)?;

                    output.push(based1);
                    output.push(based2);
                }
                Expression::NotEqualTo => {
                    based1[i] = off(interval_dto, OffVersion::IntervalRight); // TODO: This was 0 in the OG code, which meant no transformation, I think that's a bug

                    output.push(based1);
                }
                Expression::Interval => {
                    based1[i] = out(interval_dto, OutVersion::IntervalRight)?;
                    based2[i] = out(interval_dto, OutVersion::IntervalLeft)?;
                    based3[i] = off(interval_dto, OffVersion::IntervalRight);
                    based4[i] = off(interval_dto, OffVersion::IntervalLeft);

                    output.push(based1);
                    output.push(based4);
                    output.push(based3);
                    output.push(based2);
                }
            },
        }
    }

    Ok(output)
}

enum OnVersion {
    IntervalEqual,
    IntervalRight,
    IntervalLeft,
}

fn on(input: &IntervalDTO, version: OnVersion) -> Output {
    let interval = match version {
        // ==
        OnVersion::IntervalEqual => input.interval,
        // <, <=, Interval Right
        OnVersion::IntervalRight => Interval::new_closed_point(
            input.interval.hi
                - if input.interval.hi_boundary == Boundary::Open {
                    1.0
                } else {
                    0.0
                } * input.precision,
        ),
        // >, >=, Interval left
        OnVersion::IntervalLeft => Interval::new_closed_point(
            input.interval.lo
                + if input.interval.lo_boundary == Boundary::Open {
                    1.0
                } else {
                    0.0
                } * input.precision,
        ),
    };

    Output::Interval(interval)
}

enum InVersion {
    IntervalRight,
    IntervalLeft,
    Interval,
}

fn calc_in(input: &IntervalDTO, version: InVersion) -> Result<Output, IntervalError> {
    let interval = match version {
        // <, <=
        InVersion::IntervalRight => Interval::new_closed(
            f32::NEG_INFINITY,
            input.interval.hi
                - if input.interval.hi_boundary == Boundary::Open {
                    1.0
                } else {
                    0.0
                } * input.precision,
        )?,
        // >, >=
        InVersion::IntervalLeft => Interval::new_closed(
            input.interval.lo
                + if input.interval.lo_boundary == Boundary::Open {
                    1.0
                } else {
                    0.0
                } * input.precision,
            f32::INFINITY,
        )?,
        // Interval
        InVersion::Interval => Interval::new_closed(
            input.interval.lo
                + if input.interval.lo_boundary == Boundary::Open {
                    1.0
                } else {
                    0.0
                } * input.precision,
            input.interval.hi
                - if input.interval.hi_boundary == Boundary::Open {
                    1.0
                } else {
                    0.0
                } * input.precision,
        )?,
    };

    Ok(Output::Interval(interval))
}

enum InInVersion {
    IntervalRight,
    IntervalLeft,
}

fn in_in(input: &IntervalDTO, version: InInVersion) -> Result<Output, IntervalError> {
    let interval = match version {
        // <, <=
        InInVersion::IntervalRight => Interval::new_closed(
            f32::NEG_INFINITY,
            input.interval.hi
                - if input.interval.hi_boundary == Boundary::Open {
                    2.0
                } else {
                    1.0
                } * input.precision,
        )?,
        // >, >=
        InInVersion::IntervalLeft => Interval::new_closed(
            input.interval.lo
                + if input.interval.lo_boundary == Boundary::Open {
                    2.0
                } else {
                    1.0
                } * input.precision,
            f32::INFINITY,
        )?,
    };

    Ok(Output::Interval(interval))
}
enum OffVersion {
    IntervalRight,
    IntervalLeft,
}

fn off(input: &IntervalDTO, version: OffVersion) -> Output {
    let interval = match version {
        // <, <=, Interval Right, == right
        OffVersion::IntervalRight => Interval::new_closed_point(
            input.interval.hi
                + if input.interval.hi_boundary == Boundary::Open {
                    0.0
                } else {
                    1.0
                } * input.precision,
        ),
        // >, >=, // Interval left, == left
        OffVersion::IntervalLeft => Interval::new_closed_point(
            input.interval.lo
                - if input.interval.lo_boundary == Boundary::Open {
                    0.0
                } else {
                    1.0
                } * input.precision,
        ),
    };

    Output::Interval(interval)
}

enum OutVersion {
    IntervalRight,
    IntervalLeft,
    Right,
    Left,
}

fn out(input: &IntervalDTO, version: OutVersion) -> Result<Output, IntervalError> {
    let interval = match version {
        // <, <=, Interval Right
        OutVersion::IntervalRight => Interval::new_closed(
            input.interval.hi
                + if input.interval.hi_boundary == Boundary::Open {
                    1.0
                } else {
                    2.0
                } * input.precision,
            f32::INFINITY,
        )?,
        // >, =>, Interval Left
        OutVersion::IntervalLeft => Interval::new_closed(
            f32::NEG_INFINITY,
            input.interval.lo
                - if input.interval.lo_boundary == Boundary::Open {
                    1.0
                } else {
                    2.0
                } * input.precision,
        )?,
        // =, Right
        OutVersion::Right => {
            Interval::new_closed(input.interval.hi + input.precision, f32::INFINITY)?
        }
        // =, Left
        OutVersion::Left => {
            Interval::new_closed(f32::NEG_INFINITY, input.interval.lo - input.precision)?
        }
    };

    Ok(Output::Interval(interval))
}

#[cfg(test)]
mod tests {
    use crate::{
        dto::{
            BoolDTO, BoolExpression, Expression, Input, IntervalDTO, NTupleInput, NTupleOutput,
            Output,
        },
        interval::{Boundary, Interval},
    };

    use super::generate_test_cases_for_inputs;

    #[test]
    fn test_generate_test_cases_for_inputs() {
        // true;   <50; *
        let inputs: Vec<Input> = vec![
            Input::Bool(BoolDTO {
                expression: BoolExpression::IsTrue,
                bool_val: true,
                is_constant: false,
            }),
            Input::Interval(IntervalDTO {
                expression: Expression::LessThan,
                interval: Interval::new(Boundary::Open, f32::NEG_INFINITY, 50.0, Boundary::Open)
                    .unwrap(),
                precision: 0.01,
                is_constant: false,
            }),
            Input::MissingVariable,
        ];

        let expected: Vec<NTupleOutput> = vec![
            NTupleOutput {
                outputs: vec![
                    Output::Bool(true),
                    Output::Interval(Interval::new_closed(f32::NEG_INFINITY, 49.98).unwrap()),
                    Output::MissingVariable,
                ],
            },
            NTupleOutput {
                outputs: vec![
                    Output::Bool(true),
                    Output::Interval(Interval::new_closed_point(49.99)),
                    Output::MissingVariable,
                ],
            },
            NTupleOutput {
                outputs: vec![
                    Output::Bool(false),
                    Output::Interval(Interval::new_closed(f32::NEG_INFINITY, 49.99).unwrap()),
                    Output::MissingVariable,
                ],
            },
            NTupleOutput {
                outputs: vec![
                    Output::Bool(true),
                    Output::Interval(Interval::new_closed(50.01, f32::INFINITY).unwrap()),
                    Output::MissingVariable,
                ],
            },
            NTupleOutput {
                outputs: vec![
                    Output::Bool(true),
                    Output::Interval(Interval::new_closed_point(50.0)),
                    Output::MissingVariable,
                ],
            },
        ];

        let result = generate_test_cases_for_inputs(&inputs).unwrap();

        assert_eq!(result, expected);
    }
}
