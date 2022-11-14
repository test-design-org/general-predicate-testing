use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::format;
use std::num::ParseIntError;

use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::character::complete::anychar;
use nom::character::complete::{alphanumeric0, digit1, space0};
use nom::character::is_alphabetic;
use nom::combinator::fail;
use nom::combinator::{complete, cond, map, map_res, opt, peek, recognize};
use nom::error;
use nom::error::ParseError;
use nom::multi::count;
use nom::multi::many0;
use nom::multi::separated_list1;
use nom::number::complete::recognize_float;
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;
use nom::Parser;
use nom::{branch::alt, character::streaming::char};

use crate::interval;
use crate::interval::{Boundary, MultiInterval};

use self::ast::BinaryCondition;
use self::ast::BoolCondition;
use self::ast::ConstantPosition;
use self::ast::IntervalCondition;
use self::ast::{BinaryOp, BoolOp, Condition, EqOp, IntervalOp};

pub mod ast;

fn comment(input: &str) -> IResult<&str, ()> {
    todo!("Implement comment parsing")
}

fn whitespace(input: &str) -> IResult<&str, ()> {
    // many0(alt((space0(input), comment)))(input)
    map(space0, |_| ())(input)
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

pub fn interval(input: &str) -> IResult<&str, MultiInterval> {
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
    )(input)
}

fn parse_alphabetic(input: &str) -> IResult<&str, char> {
    let (i, c) = anychar(input)?;
    if is_alphabetic(c as u8) {
        Ok((i, c))
    } else {
        fail(input)
    }
}

fn keywords() -> HashSet<&'static str> {
    HashSet::from(["if", "else", "true", "false"])
}

fn var_name(input: &str) -> IResult<&str, &str> {
    map_res(
        recognize(tuple((alt((parse_alphabetic, char('_'))), alphanumeric0))),
        |var_name| {
            if keywords().contains(var_name) {
                Err(format!(
                    "Cannot name a variable '{var_name}', it is a reserved keyword!"
                ))
            } else {
                Ok(var_name)
            }
        },
    )(input)
}

fn condition_bool_lhs(input: &str) -> IResult<&str, Condition> {
    map(
        tuple((
            terminated(boolean, whitespace),
            terminated(eq_op, whitespace),
            terminated(var_name, whitespace),
        )),
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
        tuple((
            terminated(var_name, whitespace),
            terminated(eq_op, whitespace),
            terminated(boolean, whitespace),
        )),
        |(var_name, eq_op, constant)| {
            Condition::Bool(BoolCondition {
                var_name,
                constant,
                eq_op,
            })
        },
    )(input)
}

fn condition_binary_lhs(input: &str) -> IResult<&str, Condition> {
    map(
        tuple((
            terminated(number, whitespace),
            terminated(binary_op, whitespace),
            terminated(var_name, whitespace),
        )),
        |(constant, binary_op, var_name)| {
            Condition::Binary(BinaryCondition {
                var_name,
                constant,
                constant_position: ConstantPosition::LeftHandSide,
                binary_op: binary_op.flip(),
            })
        },
    )(input)
}

fn condition_binary_rhs(input: &str) -> IResult<&str, Condition> {
    map(
        tuple((
            terminated(var_name, whitespace),
            terminated(binary_op, whitespace),
            terminated(number, whitespace),
        )),
        |(var_name, binary_op, constant)| {
            Condition::Binary(BinaryCondition {
                var_name,
                constant,
                constant_position: ConstantPosition::RightHandSide,
                binary_op,
            })
        },
    )(input)
}

fn condition_interval(input: &str) -> IResult<&str, Condition> {
    map(
        tuple((
            terminated(var_name, whitespace),
            terminated(interval_op, whitespace),
            terminated(interval, whitespace),
        )),
        |(var_name, interval_op, interval)| {
            Condition::Interval(IntervalCondition {
                var_name,
                interval_op,
                interval,
            })
        },
    )(input)
}

fn condition(input: &str) -> IResult<&str, Condition> {
    alt((
        condition_binary_lhs,
        condition_binary_rhs,
        condition_bool_lhs,
        condition_bool_rhs,
        condition_interval,
    ))(input)
}

fn conditions(input: &str) -> IResult<&str, Vec<Condition>> {
    separated_list1(terminated(bool_op, whitespace), condition)(input)
}

// TODO: if-elseif-else, vardecl, feature

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace("     "), Ok(("", ())));
        assert_eq!(whitespace("     asd "), Ok(("asd ", ())));
        assert_eq!(whitespace("qwe"), Ok(("qwe", ())));
        assert_eq!(whitespace(""), Ok(("", ())));
    }

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
        assert!(var_name("9asd").is_err());

        assert!(var_name("if").is_err());
        assert!(var_name("else").is_err());
        assert!(var_name("true").is_err());
        assert!(var_name("false").is_err());
    }

    #[test]
    fn test_condition_bool_lhs() {
        assert_eq!(
            condition_bool_lhs("true = x"),
            Ok((
                "",
                Condition::Bool(BoolCondition {
                    var_name: "x",
                    constant: true,
                    eq_op: EqOp::Equal
                })
            ))
        );
        assert_eq!(
            condition_bool_lhs("false=foo"),
            Ok((
                "",
                Condition::Bool(BoolCondition {
                    var_name: "foo",
                    constant: false,
                    eq_op: EqOp::Equal
                })
            ))
        );
        assert_eq!(
            condition_bool_lhs("false!=foo"),
            Ok((
                "",
                Condition::Bool(BoolCondition {
                    var_name: "foo",
                    constant: false,
                    eq_op: EqOp::NotEqual
                })
            ))
        );
        assert_eq!(
            condition_bool_lhs("true   =   y) asd"),
            Ok((
                ") asd",
                Condition::Bool(BoolCondition {
                    var_name: "y",
                    constant: true,
                    eq_op: EqOp::Equal
                })
            ))
        );
        assert!(condition_bool_lhs("9asd").is_err());
        assert!(condition_bool_lhs("y not foo").is_err());
        assert!(condition_bool_lhs("false = false").is_err());
        assert!(condition_bool_lhs("true = true").is_err());
        assert!(condition_bool_lhs("x = true").is_err());
        assert!(condition_bool_lhs("false =").is_err());
    }

    #[test]
    fn test_condition_bool_rhs() {
        assert_eq!(
            condition_bool_rhs("x = true"),
            Ok((
                "",
                Condition::Bool(BoolCondition {
                    var_name: "x",
                    constant: true,
                    eq_op: EqOp::Equal
                })
            ))
        );
        assert_eq!(
            condition_bool_rhs("foo=false"),
            Ok((
                "",
                Condition::Bool(BoolCondition {
                    var_name: "foo",
                    constant: false,
                    eq_op: EqOp::Equal
                })
            ))
        );
        assert_eq!(
            condition_bool_rhs("foo!=false"),
            Ok((
                "",
                Condition::Bool(BoolCondition {
                    var_name: "foo",
                    constant: false,
                    eq_op: EqOp::NotEqual
                })
            ))
        );
        assert_eq!(
            condition_bool_rhs("y   =   true) asd"),
            Ok((
                ") asd",
                Condition::Bool(BoolCondition {
                    var_name: "y",
                    constant: true,
                    eq_op: EqOp::Equal
                })
            ))
        );
        assert!(condition_bool_rhs("9asd").is_err());
        assert!(condition_bool_rhs("y not foo").is_err());
        assert!(condition_bool_rhs("false = false").is_err());
        assert!(condition_bool_rhs("true = true").is_err());
        assert!(condition_bool_rhs("true = x").is_err());
        assert!(condition_bool_rhs("x =").is_err());
    }

    #[test]
    fn test_condition_binary_lhs() {
        assert_eq!(
            condition_binary_lhs("10.32 = x"),
            Ok((
                "",
                Condition::Binary(BinaryCondition {
                    var_name: "x",
                    constant_position: ConstantPosition::LeftHandSide,
                    constant: 10.32,
                    binary_op: BinaryOp::Equal
                })
            ))
        );
        assert_eq!(
            condition_binary_lhs("3<=foo"),
            Ok((
                "",
                Condition::Binary(BinaryCondition {
                    var_name: "foo",
                    constant_position: ConstantPosition::LeftHandSide,
                    constant: 3.0,
                    binary_op: BinaryOp::GreaterThanEqualTo
                })
            ))
        );
        assert_eq!(
            condition_binary_lhs("0.1   >    qwe; asd"),
            Ok((
                "; asd",
                Condition::Binary(BinaryCondition {
                    var_name: "qwe",
                    constant_position: ConstantPosition::LeftHandSide,
                    constant: 0.1,
                    binary_op: BinaryOp::LessThan
                })
            ))
        );
        assert!(condition_binary_lhs("9asd").is_err());
        assert!(condition_binary_lhs("y not foo").is_err());
        assert!(condition_binary_lhs("x = 123.0").is_err());
        assert!(condition_binary_lhs("true = x").is_err());
        assert!(condition_binary_lhs("123 = 123").is_err());
        assert!(condition_binary_lhs("123.0 >=").is_err());
    }

    #[test]
    fn test_condition_binary_rhs() {
        assert_eq!(
            condition_binary_rhs("x = 10.32"),
            Ok((
                "",
                Condition::Binary(BinaryCondition {
                    var_name: "x",
                    constant_position: ConstantPosition::RightHandSide,
                    constant: 10.32,
                    binary_op: BinaryOp::Equal
                })
            ))
        );
        assert_eq!(
            condition_binary_rhs("foo<=3"),
            Ok((
                "",
                Condition::Binary(BinaryCondition {
                    var_name: "foo",
                    constant_position: ConstantPosition::RightHandSide,
                    constant: 3.0,
                    binary_op: BinaryOp::LessThanEqualTo
                })
            ))
        );
        assert_eq!(
            condition_binary_rhs("qwe   <=    0.1; asd"),
            Ok((
                "; asd",
                Condition::Binary(BinaryCondition {
                    var_name: "qwe",
                    constant_position: ConstantPosition::RightHandSide,
                    constant: 0.1,
                    binary_op: BinaryOp::LessThanEqualTo
                })
            ))
        );
        assert!(condition_binary_rhs("9asd").is_err());
        assert!(condition_binary_rhs("y not foo").is_err());
        assert!(condition_binary_rhs("123.0 = x").is_err());
        assert!(condition_binary_rhs("x = true").is_err());
        assert!(condition_binary_rhs("123 = 123").is_err());
        assert!(condition_binary_rhs("x >=").is_err());
    }

    #[test]
    fn test_condition_interval() {
        assert_eq!(
            condition_interval("x in [0, 10]"),
            Ok((
                "",
                Condition::Interval(IntervalCondition {
                    var_name: "x",
                    interval_op: IntervalOp::In,
                    interval: MultiInterval::new(Boundary::Closed, 0.0, 10.0, Boundary::Closed)
                        .unwrap()
                })
            ))
        );
        assert_eq!(
            condition_interval("y in[0, 10]  asd"),
            Ok((
                "asd",
                Condition::Interval(IntervalCondition {
                    var_name: "y",
                    interval_op: IntervalOp::In,
                    interval: MultiInterval::new(Boundary::Closed, 0.0, 10.0, Boundary::Closed)
                        .unwrap()
                })
            ))
        );
        assert_eq!(
            condition_interval("foo not in(0,0)) qwe"),
            Ok((
                ") qwe",
                Condition::Interval(IntervalCondition {
                    var_name: "foo",
                    interval_op: IntervalOp::NotIn,
                    interval: MultiInterval::new(Boundary::Open, 0.0, 0.0, Boundary::Open).unwrap()
                })
            ))
        );
        assert!(condition_interval("xin[0,10]").is_err());
        assert!(condition_interval("x not [0,10]").is_err());
        assert!(condition_interval("x not in [0,").is_err());
        assert!(condition_interval("[0, 10]").is_err());
        assert!(condition_interval("in [0, 10]").is_err());
        assert!(condition_interval(" in [0, 10]").is_err());
    }

    #[test]
    fn test_condition() {
        assert_eq!(
            condition("true = x"),
            Ok((
                "",
                Condition::Bool(BoolCondition {
                    var_name: "x",
                    constant: true,
                    eq_op: EqOp::Equal
                })
            ))
        );
        assert_eq!(
            condition("x = true"),
            Ok((
                "",
                Condition::Bool(BoolCondition {
                    var_name: "x",
                    constant: true,
                    eq_op: EqOp::Equal
                })
            ))
        );
        assert_eq!(
            condition("10.32 = x"),
            Ok((
                "",
                Condition::Binary(BinaryCondition {
                    var_name: "x",
                    constant_position: ConstantPosition::LeftHandSide,
                    constant: 10.32,
                    binary_op: BinaryOp::Equal
                })
            ))
        );
        assert_eq!(
            condition("x = 10.32"),
            Ok((
                "",
                Condition::Binary(BinaryCondition {
                    var_name: "x",
                    constant_position: ConstantPosition::RightHandSide,
                    constant: 10.32,
                    binary_op: BinaryOp::Equal
                })
            ))
        );
        assert_eq!(
            condition("x in [0, 10]"),
            Ok((
                "",
                Condition::Interval(IntervalCondition {
                    var_name: "x",
                    interval_op: IntervalOp::In,
                    interval: MultiInterval::new(Boundary::Closed, 0.0, 10.0, Boundary::Closed)
                        .unwrap()
                })
            ))
        );
    }

    #[test]
    fn test_conditions() {
        let x_eq_true = Condition::Bool(BoolCondition {
            var_name: "x",
            constant: true,
            eq_op: EqOp::Equal,
        });

        let y_greater_0 = Condition::Binary(BinaryCondition {
            var_name: "y",
            constant_position: ConstantPosition::LeftHandSide,
            constant: 0.0,
            binary_op: BinaryOp::GreaterThan,
        });

        assert_eq!(conditions("true = x"), Ok(("", vec![x_eq_true.clone()])));
        assert_eq!(
            conditions("true = x && 0 < y"),
            Ok(("", vec![x_eq_true.clone(), y_greater_0.clone()]))
        );
        assert_eq!(
            conditions("true = x && 0 < y && 0 < y    asd"),
            Ok((
                "asd",
                vec![x_eq_true.clone(), y_greater_0.clone(), y_greater_0.clone()]
            ))
        );
        assert!(conditions("").is_err());
        assert!(conditions("true = x &&").is_err());
    }
}
