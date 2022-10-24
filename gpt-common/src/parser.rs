use std::collections::HashMap;
use std::num::ParseIntError;

use nom::bytes::complete::tag;
use nom::character::complete::{digit1, space0};
use nom::combinator::{complete, map, map_res, opt, recognize};
use nom::number::complete::recognize_float;
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;
use nom::{branch::alt, character::streaming::char};

use crate::interval::{Interval, Openness};

use self::ast::{BinaryOp, BoolOp, EqOp, IntervalOp};

pub mod ast;

fn comment(input: &str) -> IResult<&str, ()> {
    todo!("Implement comment parsing")
}

fn float(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize(tuple((opt(char('-')), digit1, char('.'), digit1))),
        str::parse,
    )(input)
}

fn int(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize(tuple((opt(char('-')), digit1))),
        |x: &str| -> Result<f32, ParseIntError> {
            let int = x.parse::<i32>()?;
            Ok(int as f32)
        },
    )(input)
}

fn infinity(input: &str) -> IResult<&str, f32> {
    map(tuple((opt(char('-')), tag("Inf"))), |(minus, inf)| {
        if minus.is_none() {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        }
    })(input)
}

fn number(input: &str) -> IResult<&str, f32> {
    alt((complete(float), int, infinity))(input)
}

fn boolean(input: &str) -> IResult<&str, bool> {
    map_res(alt((tag("true"), tag("false"))), |x| match x {
        "true" => Ok(true),
        "false" => Ok(false),
        &_ => Err(()),
    })(input)
}

fn eq_op(input: &str) -> IResult<&str, EqOp> {
    map_res(alt((tag("="), tag("!="))), |x| match x {
        "=" => Ok(EqOp::Equal),
        "!=" => Ok(EqOp::NotEqual),
        &_ => Err(()),
    })(input)
}

fn interval_op(input: &str) -> IResult<&str, IntervalOp> {
    map_res(alt((tag("in"), tag("not in"))), |x| match x {
        "in" => Ok(IntervalOp::In),
        "not in" => Ok(IntervalOp::NotIn),
        &_ => Err(()),
    })(input)
}

fn binary_op(input: &str) -> IResult<&str, BinaryOp> {
    map_res(
        alt((
            tag("<="),
            tag(">="),
            tag("!="),
            tag("<"),
            tag(">"),
            tag("="),
        )),
        |x| match x {
            "<=" => Ok(BinaryOp::LessThanEqualTo),
            ">=" => Ok(BinaryOp::GreaterThanEqualTo),
            "!=" => Ok(BinaryOp::NotEqual),
            "<" => Ok(BinaryOp::LessThan),
            ">" => Ok(BinaryOp::GreaterThan),
            "=" => Ok(BinaryOp::Equal),
            &_ => Err(()),
        },
    )(input)
}

fn bool_op(input: &str) -> IResult<&str, BoolOp> {
    map_res(
        alt((
            tag("&&"),
            // tag("||"),
        )),
        |x| {
            match x {
                "&&" => Ok(BoolOp::And),
                // "||" => BoolOp::Or,
                &_ => Err(()),
            }
        },
    )(input)
}

fn parse_lo_openness(input: &str) -> IResult<&str, Openness> {
    map_res(alt((char('('), char('['))), |left_brace| match left_brace {
        '(' => Ok(Openness::Open),
        '[' => Ok(Openness::Closed),
        _ => Err(()),
    })(input)
}

fn parse_hi_openness(input: &str) -> IResult<&str, Openness> {
    map_res(alt((char(')'), char(']'))), |left_brace| match left_brace {
        ')' => Ok(Openness::Open),
        ']' => Ok(Openness::Closed),
        _ => Err(()),
    })(input)
}

fn interval(input: &str) -> IResult<&str, Interval> {
    map_res(
        tuple((
            terminated(parse_lo_openness, space0),
            terminated(number, space0),
            terminated(char(','), space0),
            terminated(number, space0),
            terminated(parse_hi_openness, space0),
        )),
        |(lo_openness, lo, _comma, hi, hi_openness)| {
            Ok::<Interval, ()>(Interval::new(lo_openness, lo, hi, hi_openness))
        },
    )(input)
}

// TODO: Cond, Conditions, if-elseif-else, vardecl, feature

#[cfg(test)]
mod tests {
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
        assert_eq!(number("123."), Ok((".", 123.0)));
        assert_eq!(number("-Inf"), Ok(("", f32::NEG_INFINITY)));
        assert_eq!(number("Inf"), Ok(("", f32::INFINITY)));
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
        assert_eq!(eq_op("="), Ok(("", EqOp::Equal)));
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
        assert_eq!(binary_op("="), Ok(("", BinaryOp::Equal)));
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
        assert_eq!(parse_lo_openness("("), Ok(("", Openness::Open)));
        assert_eq!(parse_lo_openness("["), Ok(("", Openness::Closed)));
        assert!(parse_lo_openness("other").is_err());
    }

    #[test]
    fn test_parse_hi_openness() {
        assert_eq!(parse_hi_openness(")"), Ok(("", Openness::Open)));
        assert_eq!(parse_hi_openness("]"), Ok(("", Openness::Closed)));
        assert!(parse_hi_openness("other").is_err());
    }

    #[test]
    fn test_interval() {
        assert_eq!(
            interval("(12.0,Inf]"),
            Ok((
                "",
                Interval::new(Openness::Open, 12.0, f32::INFINITY, Openness::Closed)
            ))
        );
        assert_eq!(
            interval("[   -43   ,    54   )   "),
            Ok((
                "",
                Interval::new(Openness::Closed, -43.0, 54.0, Openness::Open)
            ))
        );
        assert!(interval("other").is_err());
    }
}
