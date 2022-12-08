use crate::dto::{BoolDTO, BoolExpression, Expression};
use crate::interval::Interval;
use crate::util::UniquesVec;

use crate::{
    dto::{Input, IntervalDTO, NTuple},
    interval::{Boundary, IntervalError},
};

pub fn generate_test_cases_for_multiple_features(
    features: &Vec<Vec<NTuple>>,
) -> Result<Vec<NTuple>, IntervalError> {
    let mut res = Vec::new();
    for feature in features {
        let mut test_cases = generate_test_cases_for_feature(feature)?;
        res.append(&mut test_cases);
    }
    Ok(res)
}

fn generate_test_cases_for_feature(nTuples: &Vec<NTuple>) -> Result<Vec<NTuple>, IntervalError> {
    let inputs = nTuples.into_iter().flat_map(|x| x.inputs.clone()).collect();
    generate_test_cases_for_inputs(&inputs)
}

fn generate_test_cases_for_inputs(inputs: &Vec<Input>) -> Result<Vec<NTuple>, IntervalError> {
    let mut modifiedInputs = vec![
        calculate_in_on_patterns1(inputs)?,
        calculate_in_on_patterns2(inputs)?,
    ];
    modifiedInputs.extend(off_out(inputs)?);

    let mut nTuples: Vec<NTuple> = modifiedInputs
        .into_iter()
        .map(|inputs| NTuple { inputs })
        .collect();

    nTuples = nTuples.uniques();

    Ok(nTuples)
}

fn calculate_in_on_patterns1(inputs: &Vec<Input>) -> Result<Vec<Input>, IntervalError> {
    inputs
        .iter()
        .map(|input| match input {
            Input::MissingVariable => Ok(input.clone()),
            Input::Bool(_) => Ok(input.clone()),
            Input::Interval(IntervalDTO { is_constant, .. }) if *is_constant => Ok(input.clone()),
            Input::Interval(interval_dto) => match interval_dto.expression {
                Expression::LessThan | Expression::LessThanOrEqualTo => {
                    in_in(interval_dto, InInVersion::IntervalRight)
                }

                Expression::EqualTo => on(interval_dto, OnVersion::IntervalEqual),

                Expression::GreaterThan | Expression::GreaterThanOrEqualTo => {
                    in_in(interval_dto, InInVersion::IntervalLeft)
                }

                Expression::NotEqualTo => In(interval_dto, InVersion::IntervalRight),
                Expression::Interval => on(interval_dto, OnVersion::IntervalLeft),
            },
        })
        .collect()
}

fn calculate_in_on_patterns2(inputs: &Vec<Input>) -> Result<Vec<Input>, IntervalError> {
    inputs
        .iter()
        .map(|input| match input {
            Input::MissingVariable => Ok(input.clone()),
            Input::Bool(_) => Ok(input.clone()),
            Input::Interval(IntervalDTO { is_constant, .. }) if *is_constant => Ok(input.clone()),
            Input::Interval(interval_dto) => match interval_dto.expression {
                Expression::LessThan | Expression::LessThanOrEqualTo => {
                    on(interval_dto, OnVersion::IntervalRight)
                }

                Expression::EqualTo => on(interval_dto, OnVersion::IntervalEqual),

                Expression::GreaterThan | Expression::GreaterThanOrEqualTo => {
                    on(interval_dto, OnVersion::IntervalLeft)
                }

                Expression::NotEqualTo => In(interval_dto, InVersion::IntervalLeft),
                Expression::Interval => on(interval_dto, OnVersion::IntervalRight),
            },
        })
        .collect()
}

fn baseline(inputs: &Vec<Input>) -> Result<Vec<Input>, IntervalError> {
    inputs
        .iter()
        .map(|input| -> Result<Input, IntervalError> {
            match input {
                Input::MissingVariable => Ok(input.clone()),
                Input::Bool(_) => Ok(input.clone()),
                Input::Interval(interval_dto) => match interval_dto.expression {
                    Expression::LessThan | Expression::LessThanOrEqualTo => {
                        In(interval_dto, InVersion::IntervalRight)
                    }

                    Expression::GreaterThan
                    | Expression::GreaterThanOrEqualTo
                    | Expression::NotEqualTo => In(interval_dto, InVersion::IntervalLeft),

                    Expression::EqualTo => on(interval_dto, OnVersion::IntervalEqual),
                    Expression::Interval => In(interval_dto, InVersion::Interval),
                },
            }
        })
        .collect()
}

fn off_out(inputs: &Vec<Input>) -> Result<Vec<Vec<Input>>, IntervalError> {
    let mut output: Vec<Vec<Input>> = Vec::new();

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
                    based1[i] = Input::Bool(BoolDTO {
                        expression: expression.clone(),
                        bool_val: false,
                        is_constant: false,
                    });
                    output.push(based1);
                }
                BoolExpression::IsFalse => {
                    based1[i] = Input::Bool(BoolDTO {
                        expression: expression.clone(),
                        bool_val: true,
                        is_constant: false,
                    });
                    output.push(based1);
                }
            },
            Input::Interval(interval_dto @ IntervalDTO { expression, .. }) => match expression {
                Expression::LessThan | Expression::LessThanOrEqualTo => {
                    based1[i] = out(interval_dto, OutVersion::IntervalRight)?;
                    based2[i] = off(interval_dto, OffVersion::IntervalRight)?;

                    output.push(based1);
                    output.push(based2);
                }
                Expression::GreaterThan | Expression::GreaterThanOrEqualTo => {
                    based1[i] = out(interval_dto, OutVersion::IntervalLeft)?;
                    based2[i] = off(interval_dto, OffVersion::IntervalLeft)?;

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
                    based1[i] = off(interval_dto, OffVersion::IntervalRight)?; // TODO: This was 0 in the OG code, which meant no transformation, I think that's a bug

                    output.push(based1);
                }
                Expression::Interval => {
                    based1[i] = out(interval_dto, OutVersion::IntervalRight)?;
                    based2[i] = out(interval_dto, OutVersion::IntervalLeft)?;
                    based3[i] = off(interval_dto, OffVersion::IntervalRight)?;
                    based4[i] = off(interval_dto, OffVersion::IntervalLeft)?;

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

fn on(input: &IntervalDTO, version: OnVersion) -> Result<Input, IntervalError> {
    let interval = match version {
        // ==
        OnVersion::IntervalEqual => input.interval.clone(),
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

    Ok(Input::Interval(IntervalDTO {
        expression: input.expression,
        interval,
        precision: input.precision,
        is_constant: false,
    }))
}

enum InVersion {
    IntervalRight,
    IntervalLeft,
    Interval,
}

fn In(input: &IntervalDTO, version: InVersion) -> Result<Input, IntervalError> {
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

    Ok(Input::Interval(IntervalDTO {
        expression: input.expression,
        interval,
        precision: input.precision,
        is_constant: false,
    }))
}

enum InInVersion {
    IntervalRight,
    IntervalLeft,
}

fn in_in(input: &IntervalDTO, version: InInVersion) -> Result<Input, IntervalError> {
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

    Ok(Input::Interval(IntervalDTO {
        expression: input.expression,
        interval,
        precision: input.precision,
        is_constant: false,
    }))
}
enum OffVersion {
    IntervalRight,
    IntervalLeft,
}

fn off(input: &IntervalDTO, version: OffVersion) -> Result<Input, IntervalError> {
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

    Ok(Input::Interval(IntervalDTO {
        expression: input.expression,
        interval,
        precision: input.precision,
        is_constant: false,
    }))
}

enum OutVersion {
    IntervalRight,
    IntervalLeft,
    Right,
    Left,
}

fn out(input: &IntervalDTO, version: OutVersion) -> Result<Input, IntervalError> {
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

    Ok(Input::Interval(IntervalDTO {
        expression: input.expression,
        interval,
        precision: input.precision,
        is_constant: false,
    }))
}

#[cfg(test)]
mod tests {
    use crate::{
        dto::{BoolDTO, BoolExpression, Expression, Input, IntervalDTO, NTuple},
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

        let expected: Vec<NTuple> = vec![
            NTuple {
                inputs: vec![
                    Input::Bool(BoolDTO {
                        expression: BoolExpression::IsTrue,
                        bool_val: true,
                        is_constant: false,
                    }),
                    Input::Interval(IntervalDTO {
                        expression: Expression::LessThan,
                        interval: Interval::new_closed(f32::NEG_INFINITY, 49.98).unwrap(),
                        precision: 0.01,
                        is_constant: false,
                    }),
                    Input::MissingVariable,
                ],
            },
            NTuple {
                inputs: vec![
                    Input::Bool(BoolDTO {
                        expression: BoolExpression::IsTrue,
                        bool_val: true,
                        is_constant: false,
                    }),
                    Input::Interval(IntervalDTO {
                        expression: Expression::LessThan,
                        interval: Interval::new_closed_point(49.99),
                        precision: 0.01,
                        is_constant: false,
                    }),
                    Input::MissingVariable,
                ],
            },
            NTuple {
                inputs: vec![
                    Input::Bool(BoolDTO {
                        expression: BoolExpression::IsTrue,
                        bool_val: false,
                        is_constant: false,
                    }),
                    Input::Interval(IntervalDTO {
                        expression: Expression::LessThan,
                        interval: Interval::new_closed(f32::NEG_INFINITY, 49.99).unwrap(),
                        precision: 0.01,
                        is_constant: false,
                    }),
                    Input::MissingVariable,
                ],
            },
            NTuple {
                inputs: vec![
                    Input::Bool(BoolDTO {
                        expression: BoolExpression::IsTrue,
                        bool_val: true,
                        is_constant: false,
                    }),
                    Input::Interval(IntervalDTO {
                        expression: Expression::LessThan,
                        interval: Interval::new_closed(50.01, f32::INFINITY).unwrap(),
                        precision: 0.01,
                        is_constant: false,
                    }),
                    Input::MissingVariable,
                ],
            },
            NTuple {
                inputs: vec![
                    Input::Bool(BoolDTO {
                        expression: BoolExpression::IsTrue,
                        bool_val: true,
                        is_constant: false,
                    }),
                    Input::Interval(IntervalDTO {
                        expression: Expression::LessThan,
                        interval: Interval::new_closed_point(50.0),
                        precision: 0.01,
                        is_constant: false,
                    }),
                    Input::MissingVariable,
                ],
            },
        ];

        let result = generate_test_cases_for_inputs(&inputs).unwrap();

        assert_eq!(result, expected);
    }
}
