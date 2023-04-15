use std::{collections::HashSet, num::ParseIntError};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::{anychar, digit1, space0},
        is_alphabetic, is_alphanumeric,
        streaming::char,
    },
    combinator::{complete, cut, fail, map, map_res, opt, recognize, value},
    error::context,
    multi::many0,
    sequence::{terminated, tuple},
};

use super::{
    ast::{BinaryOp, BoolOp, EqOp, IntervalOp},
    IResult,
};
use crate::interval::{Boundary, MultiInterval};

pub fn float(input: &str) -> IResult<f32> {
    context(
        "float",
        map_res(
            recognize(tuple((opt(char('-')), digit1, char('.'), cut(digit1)))),
            str::parse,
        ),
    )(input)
}

pub fn int(input: &str) -> IResult<f32> {
    map_res(
        recognize(tuple((opt(char('-')), digit1))),
        |x: &str| -> Result<f32, ParseIntError> {
            let int = x.parse::<i32>()?;
            Ok(int as f32)
        },
    )(input)
}

pub fn infinity(input: &str) -> IResult<f32> {
    map(tuple((opt(char('-')), tag("Inf"))), |(minus, _)| {
        if minus.is_none() {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        }
    })(input)
}

pub fn number(input: &str) -> IResult<f32> {
    context("number", alt((complete(float), int, infinity)))(input)
}

pub fn boolean(input: &str) -> IResult<bool> {
    context(
        "boolean",
        alt((value(true, tag("true")), value(false, tag("false")))),
    )(input)
}

pub fn eq_op(input: &str) -> IResult<EqOp> {
    context(
        "Equality operator",
        alt((
            value(EqOp::Equal, tag("==")),
            value(EqOp::NotEqual, tag("!=")),
        )),
    )(input)
}

pub fn interval_op(input: &str) -> IResult<IntervalOp> {
    context(
        "Interval Operator",
        alt((
            value(IntervalOp::In, tag("in")),
            value(IntervalOp::NotIn, tag("not in")),
        )),
    )(input)
}

pub fn binary_op(input: &str) -> IResult<BinaryOp> {
    context(
        "Binary Operator",
        alt((
            value(BinaryOp::LessThanEqualTo, tag("<=")),
            value(BinaryOp::GreaterThanEqualTo, tag(">=")),
            value(BinaryOp::NotEqual, tag("!=")),
            value(BinaryOp::LessThan, tag("<")),
            value(BinaryOp::GreaterThan, tag(">")),
            value(BinaryOp::Equal, tag("==")),
        )),
    )(input)
}

fn bool_op(input: &str) -> IResult<BoolOp> {
    context(
        "Boolean Operator",
        alt((value(BoolOp::And, tag("&&")), value(BoolOp::Or, tag("||")))),
    )(input)
}

fn parse_lo_openness(input: &str) -> IResult<Boundary> {
    alt((
        value(Boundary::Open, char('(')),
        value(Boundary::Closed, char('[')),
    ))(input)
}

fn parse_hi_openness(input: &str) -> IResult<Boundary> {
    alt((
        value(Boundary::Open, char(')')),
        value(Boundary::Closed, char(']')),
    ))(input)
}

pub fn interval(input: &str) -> IResult<MultiInterval> {
    context(
        "interval",
        map_res(
            tuple((
                terminated(parse_lo_openness, space0),
                terminated(number, space0),
                terminated(char(','), space0),
                terminated(number, space0),
                terminated(parse_hi_openness, space0),
            )),
            |(lo_openness, lo, _comma, hi, hi_openness)| {
                MultiInterval::new(lo_openness, lo, hi, hi_openness)
            },
        ),
    )(input)
}

fn parse_alphabetic(input: &str) -> IResult<char> {
    let (i, c) = anychar(input)?;
    if is_alphabetic(c as u8) {
        Ok((i, c))
    } else {
        fail(input)
    }
}

fn parse_alphanumberic_or_underscore(input: &str) -> IResult<char> {
    let (i, c) = anychar(input)?;
    if is_alphanumeric(c as u8) || c == '_' {
        Ok((i, c))
    } else {
        fail(input)
    }
}

fn keywords() -> HashSet<&'static str> {
    HashSet::from(["if", "else", "true", "false"])
}

pub fn var_name(input: &str) -> IResult<&str> {
    context(
        "Variable name",
        map_res(
            recognize(tuple((
                alt((parse_alphabetic, char('_'))),
                many0(parse_alphanumberic_or_underscore),
            ))),
            |var_name| {
                if keywords().contains(var_name) {
                    Err(format!(
                        "Cannot name a variable '{var_name}', it is a reserved keyword!"
                    ))
                } else {
                    Ok(var_name)
                }
            },
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_int() {
        assert_eq!(int("1"), Ok(("", 1.0)));
        assert_eq!(int("-1"), Ok(("", -1.0)));
        assert_eq!(int("1.0"), Ok((".0", 1.0)));
        assert_eq!(int("123"), Ok(("", 123.0)));
        assert_eq!(int("-123"), Ok(("", -123.0)));
        assert_eq!(int("123.0"), Ok((".0", 123.0)));
        assert_eq!(int("123.123"), Ok((".123", 123.0)));
        assert_eq!(int("123.123000000"), Ok((".123000000", 123.0)));
        assert_eq!(int("123."), Ok((".", 123.0)));
        assert!(int("other").is_err());
    }

    #[test]
    fn test_float() {
        assert!(float("1").is_err());
        assert_eq!(float("1.0"), Ok(("", 1.0)));
        assert_eq!(float("-1.0"), Ok(("", -1.0)));
        assert!(float("123").is_err());
        assert_eq!(float("123.0"), Ok(("", 123.0)));
        assert_eq!(float("123.123"), Ok(("", 123.123)));
        assert_eq!(float("-123.123"), Ok(("", -123.123)));
        assert_eq!(float("123.123000000"), Ok(("", 123.123)));
        assert!(float("123.").is_err());
        assert!(float("other").is_err());
    }

    #[test]
    fn test_infinity() {
        assert_eq!(infinity("-Inf"), Ok(("", f32::NEG_INFINITY)));
        assert_eq!(infinity("Inf"), Ok(("", f32::INFINITY)));
        assert!(infinity("other").is_err());
    }

    #[test]
    fn test_number() {
        assert_eq!(number("1"), Ok(("", 1.0)));
        assert_eq!(number("-1"), Ok(("", -1.0)));
        assert_eq!(number("1.0"), Ok(("", 1.0)));
        assert_eq!(number("-1.0"), Ok(("", -1.0)));
        assert_eq!(number("123"), Ok(("", 123.0)));
        assert_eq!(number("-123"), Ok(("", -123.0)));
        assert_eq!(number("123.0"), Ok(("", 123.0)));
        assert_eq!(number("-123.0"), Ok(("", -123.0)));
        assert_eq!(number("123.123"), Ok(("", 123.123)));
        assert_eq!(number("123.123000000"), Ok(("", 123.123)));
        assert_eq!(number("-Inf"), Ok(("", f32::NEG_INFINITY)));
        assert_eq!(number("Inf"), Ok(("", f32::INFINITY)));
        assert!(number("123.").is_err());
        assert!(number("other").is_err());
    }

    #[test]
    fn test_boolean() {
        assert_eq!(boolean("true"), Ok(("", true)));
        assert_eq!(boolean("false"), Ok(("", false)));
        assert!(boolean("other").is_err());
    }

    #[test]
    fn test_eq_op() {
        assert_eq!(eq_op("=="), Ok(("", EqOp::Equal)));
        assert_eq!(eq_op("!="), Ok(("", EqOp::NotEqual)));
        assert!(eq_op("other").is_err());
    }

    #[test]
    fn test_interval_op() {
        assert_eq!(interval_op("in"), Ok(("", IntervalOp::In)));
        assert_eq!(interval_op("not in"), Ok(("", IntervalOp::NotIn)));
        assert!(interval_op("other").is_err());
    }

    #[test]
    fn test_binary_op() {
        assert_eq!(binary_op("<="), Ok(("", BinaryOp::LessThanEqualTo)));
        assert_eq!(binary_op(">="), Ok(("", BinaryOp::GreaterThanEqualTo)));
        assert_eq!(binary_op("!="), Ok(("", BinaryOp::NotEqual)));
        assert_eq!(binary_op("=="), Ok(("", BinaryOp::Equal)));
        assert_eq!(binary_op("<"), Ok(("", BinaryOp::LessThan)));
        assert_eq!(binary_op(">"), Ok(("", BinaryOp::GreaterThan)));
        assert!(binary_op("other").is_err());
    }

    #[test]
    fn test_bool_op() {
        assert_eq!(bool_op("&&"), Ok(("", BoolOp::And)));
        assert!(bool_op("other").is_err());
    }

    #[test]
    fn test_parse_lo_openness() {
        assert_eq!(parse_lo_openness("("), Ok(("", Boundary::Open)));
        assert_eq!(parse_lo_openness("["), Ok(("", Boundary::Closed)));
        assert!(parse_lo_openness("other").is_err());
    }

    #[test]
    fn test_parse_hi_openness() {
        assert_eq!(parse_hi_openness(")"), Ok(("", Boundary::Open)));
        assert_eq!(parse_hi_openness("]"), Ok(("", Boundary::Closed)));
        assert!(parse_hi_openness("other").is_err());
    }

    #[test]
    fn test_interval() {
        assert_eq!(
            interval("(12.0,Inf]"),
            Ok((
                "",
                MultiInterval::new(Boundary::Open, 12.0, f32::INFINITY, Boundary::Closed).unwrap()
            ))
        );
        assert_eq!(
            interval("[   -43   ,    54   )   "),
            Ok((
                "",
                MultiInterval::new(Boundary::Closed, -43.0, 54.0, Boundary::Open).unwrap()
            ))
        );
        assert!(interval("other").is_err());
    }

    #[test]
    fn test_parse_alphabetic() {
        assert_eq!(parse_alphabetic("abc"), Ok(("bc", 'a')));
        assert_eq!(parse_alphabetic("QWE"), Ok(("WE", 'Q')));
        assert!(parse_alphabetic("_").is_err());
    }

    #[test]
    fn test_var_name() {
        assert_eq!(var_name("abc"), Ok(("", "abc")));
        assert_eq!(var_name("f"), Ok(("", "f")));
        assert_eq!(var_name("_private"), Ok(("", "_private")));
        assert_eq!(var_name("some123"), Ok(("", "some123")));
        assert_eq!(var_name("i18n"), Ok(("", "i18n")));
        assert_eq!(var_name("second_hand_price"), Ok(("", "second_hand_price")));
        assert!(var_name("9asd").is_err());

        assert!(var_name("if").is_err());
        assert!(var_name("else").is_err());
        assert!(var_name("true").is_err());
        assert!(var_name("false").is_err());
    }
}
