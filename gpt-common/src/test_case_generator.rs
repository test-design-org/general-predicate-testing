use crate::util::UniquesVec;

use crate::{
    dto::{Input, IntervalDTO, NTuple},
    interval::{Boundary, Interval, IntervalError},
};

fn generateTestCases(inputs: &Vec<Input>) -> Vec<NTuple> {
    let mut modifiedInputs = vec![
        calculateInOnPatterns1(inputs),
        calculateInOnPatterns2(inputs),
    ];
    modifiedInputs.extend(OffOut(inputs));

    let mut nTuples: Vec<NTuple> = modifiedInputs
        .into_iter()
        .map(|inputs| NTuple { inputs })
        .collect();

    nTuples = nTuples.uniques();

    nTuples
}

fn calculateInOnPatterns1(inputs: &Vec<Input>) -> Vec<Input> {
    todo!()
}

fn calculateInOnPatterns2(inputs: &Vec<Input>) -> Vec<Input> {
    todo!()
}

fn baseline(inputs: &Vec<Input>) -> Vec<Input> {
    todo!()
}

fn OffOut(inputs: &Vec<Input>) -> Vec<Vec<Input>> {
    todo!()
}

enum OnVersion {
    Zero,
    One,
    Two,
}

fn On(input: &IntervalDTO, version: OnVersion) -> Result<Input, IntervalError> {
    todo!()
}

enum InVersion {
    One,
    Two,
    Three,
}

fn In(input: &IntervalDTO, version: InVersion) -> Result<Input, IntervalError> {
    todo!()
}

enum InInVersion {
    IntervalRight,
    IntervalLeft,
}

fn InIn(input: &IntervalDTO, version: InInVersion) -> Result<Input, IntervalError> {
    todo!()
}
enum OffVersion {
    IntervalRight,
    IntervalLeft,
}

fn Off(input: &IntervalDTO, version: OffVersion) -> Result<Input, IntervalError> {
    todo!()
}

enum OutVersion {
    IntervalRight,
    IntervalLeft,
    Right,
    Left,
}

fn Out(input: &IntervalDTO, version: OutVersion) -> Result<Input, IntervalError> {
    let interval = match version {
        // <, <=, Interval Right
        OutVersion::IntervalRight => Interval::new_closed(
            input.interval.highest_hi()
                + if input.interval.highest_boundary() == Boundary::Open {
                    1.0
                } else {
                    2.0
                } * input.precision,
            f32::INFINITY,
        )?,
        // >, =>, Interval Left
        OutVersion::IntervalLeft => Interval::new_closed(
            f32::NEG_INFINITY,
            input.interval.lowest_lo()
                - if input.interval.lowest_boundary() == Boundary::Open {
                    1.0
                } else {
                    2.0
                } * input.precision,
        )?,
        // =, Right
        OutVersion::Right => {
            Interval::new_closed(input.interval.highest_hi() + input.precision, f32::INFINITY)?
        }
        // =, Left
        OutVersion::Left => Interval::new_closed(
            f32::NEG_INFINITY,
            input.interval.lowest_lo() - input.precision,
        )?,
    };

    Ok(Input::Interval(IntervalDTO {
        expression: input.expression,
        interval,
        precision: input.precision,
        is_constant: false,
    }))
}
