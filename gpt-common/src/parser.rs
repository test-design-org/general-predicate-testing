use std::collections::HashMap;
use std::num::ParseIntError;

use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::character::complete::anychar;
use nom::character::complete::{alphanumeric0, digit1, space0};
use nom::character::is_alphabetic;
use nom::combinator::fail;
use nom::combinator::{complete, cond, map, map_res, opt, peek, recognize};
use nom::multi::count;
use nom::number::complete::recognize_float;
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;
use nom::{branch::alt, character::streaming::char};

use crate::interval::{Boundary, Interval};

use self::ast::BoolCondition;
use self::ast::ConstantPosition;
use self::ast::{BinaryOp, BoolOp, Condition, EqOp, IntervalOp};

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

fn parse_lo_openness(input: &str) -> IResult<&str, Boundary> {
    map_res(alt((char('('), char('['))), |left_brace| match left_brace {
        '(' => Ok(Boundary::Open),
        '[' => Ok(Boundary::Closed),
        _ => Err(()),
    })(input)
}

fn parse_hi_openness(input: &str) -> IResult<&str, Boundary> {
    map_res(alt((char(')'), char(']'))), |left_brace| match left_brace {
        ')' => Ok(Boundary::Open),
        ']' => Ok(Boundary::Closed),
        _ => Err(()),
    })(input)
}

pub fn interval(input: &str) -> IResult<&str, Interval> {
    map_res(
        tuple((
            terminated(parse_lo_openness, space0),
            terminated(number, space0),
            terminated(char(','), space0),
            terminated(number, space0),
            terminated(parse_hi_openness, space0),
        )),
        |(lo_openness, lo, _comma, hi, hi_openness)| {
            Interval::new(lo_openness, lo, hi, hi_openness)
        },
    )(input)
}

// TODO: Cond, Conditions, if-elseif-else, vardecl, feature

fn parse_alphabetic(input: &str) -> IResult<&str, char> {
    let (i, c) = anychar(input)?;
    if is_alphabetic(c as u8) {
        Ok((i, c))
    } else {
        fail(input)
    }
}

fn var_name(input: &str) -> IResult<&str, &str> {
    recognize(tuple((alt((parse_alphabetic, char('_'))), alphanumeric0)))(input)
}

fn condition_bool_lhs(input: &str) -> IResult<&str, Condition> {
    map(
        tuple((boolean, eq_op, var_name)),
        |(constant, eq_op, var_name)| {
            Condition::Bool(BoolCondition {
                var_name,
                constant,
                eq_op,
            })
        },
    )(input)
}
fn condition_bool_rhs(input: &str) -> IResult<&str, Condition> {
    map(
        tuple((var_name, eq_op, boolean)),
        |(var_name, eq_op, constant)| {
            Condition::Bool(BoolCondition {
                var_name,
                constant,
                eq_op,
            })
        },
    )(input)
}

fn condition(input: &str) -> IResult<&str, Condition> {
    todo!()
}

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
                Interval::new(Boundary::Open, 12.0, f32::INFINITY, Boundary::Closed).unwrap()
            ))
        );
        assert_eq!(
            interval("[   -43   ,    54   )   "),
            Ok((
                "",
                Interval::new(Boundary::Closed, -43.0, 54.0, Boundary::Open).unwrap()
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
        assert!(var_name("9asd").is_err());
    }
}
