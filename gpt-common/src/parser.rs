use std::collections::HashSet;
use std::num::ParseIntError;

use nom::bytes::complete::tag;
use nom::character::complete::anychar;
use nom::character::complete::{digit1, space0};
use nom::character::is_alphabetic;
use nom::character::is_alphanumeric;
use nom::combinator::{complete, map, map_res, opt, recognize};
use nom::combinator::{cut, value};
use nom::combinator::{eof, fail};
use nom::error::{context, VerboseError};
use nom::multi::many0;
use nom::multi::many1;
use nom::multi::separated_list1;
use nom::sequence::{terminated, tuple};
use nom::{branch::alt, character::streaming::char};

use crate::interval::{Boundary, MultiInterval};

use self::ast::BinaryCondition;
use self::ast::BoolCondition;
use self::ast::ConditionsNode;
use self::ast::ConstantPosition;
use self::ast::ElseIfNode;
use self::ast::ElseNode;
use self::ast::FeatureNode;
use self::ast::IfNode;
use self::ast::IntervalCondition;
use self::ast::RootNode;
use self::ast::Type;
use self::ast::VarNode;
use self::ast::{BinaryOp, BoolOp, Condition, EqOp, IntervalOp};

use super::dto::NTupleInput;

pub mod ast;
mod ast_to_ir;
mod ir;
mod ir_to_ntuple;

type IResult<'a, O> = nom::IResult<&'a str, O, VerboseError<&'a str>>;

fn comment(input: &str) -> IResult<()> {
    todo!("Implement comment parsing")
}

fn whitespace(input: &str) -> IResult<()> {
    let one_whitespace = complete(alt((char('\n'), char(' '), char('\t'))));
    value((), many0(one_whitespace))(input)
}

fn float(input: &str) -> IResult<f32> {
    context(
        "float",
        map_res(
            recognize(tuple((opt(char('-')), digit1, char('.'), cut(digit1)))),
            str::parse,
        ),
    )(input)
}

fn int(input: &str) -> IResult<f32> {
    map_res(
        recognize(tuple((opt(char('-')), digit1))),
        |x: &str| -> Result<f32, ParseIntError> {
            let int = x.parse::<i32>()?;
            Ok(int as f32)
        },
    )(input)
}

fn infinity(input: &str) -> IResult<f32> {
    map(tuple((opt(char('-')), tag("Inf"))), |(minus, _)| {
        if minus.is_none() {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        }
    })(input)
}

fn number(input: &str) -> IResult<f32> {
    context("number", alt((complete(float), int, infinity)))(input)
}

fn boolean(input: &str) -> IResult<bool> {
    context(
        "boolean",
        map_res(alt((tag("true"), tag("false"))), |x| match x {
            "true" => Ok(true),
            "false" => Ok(false),
            &_ => Err(()),
        }),
    )(input)
}

fn eq_op(input: &str) -> IResult<EqOp> {
    context(
        "Equality operator",
        map_res(alt((tag("="), tag("!="))), |x| match x {
            "=" => Ok(EqOp::Equal), // TODO: This should be ==
            "!=" => Ok(EqOp::NotEqual),
            &_ => Err(()),
        }),
    )(input)
}

fn interval_op(input: &str) -> IResult<IntervalOp> {
    context(
        "Interval Operator",
        map_res(alt((tag("in"), tag("not in"))), |x| match x {
            "in" => Ok(IntervalOp::In),
            "not in" => Ok(IntervalOp::NotIn),
            &_ => Err(()),
        }),
    )(input)
}

fn binary_op(input: &str) -> IResult<BinaryOp> {
    context(
        "Binary Operator",
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
        ),
    )(input)
}

fn bool_op(input: &str) -> IResult<BoolOp> {
    context(
        "Boolean Operator",
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
        ),
    )(input)
}

fn parse_lo_openness(input: &str) -> IResult<Boundary> {
    map_res(alt((char('('), char('['))), |left_brace| match left_brace {
        '(' => Ok(Boundary::Open),
        '[' => Ok(Boundary::Closed),
        _ => Err(()),
    })(input)
}

fn parse_hi_openness(input: &str) -> IResult<Boundary> {
    map_res(alt((char(')'), char(']'))), |left_brace| match left_brace {
        ')' => Ok(Boundary::Open),
        ']' => Ok(Boundary::Closed),
        _ => Err(()),
    })(input)
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

fn var_name(input: &str) -> IResult<&str> {
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

fn condition_bool_lhs(input: &str) -> IResult<Condition> {
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

fn condition_bool_rhs(input: &str) -> IResult<Condition> {
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

fn condition_binary_lhs(input: &str) -> IResult<Condition> {
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
                binary_op,
            })
        },
    )(input)
}

fn condition_binary_rhs(input: &str) -> IResult<Condition> {
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

fn condition_interval(input: &str) -> IResult<Condition> {
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

fn condition(input: &str) -> IResult<Condition> {
    context(
        "condition",
        alt((
            condition_binary_lhs,
            condition_binary_rhs,
            condition_bool_lhs,
            condition_bool_rhs,
            condition_interval,
        )),
    )(input)
}

fn conditions(input: &str) -> IResult<ConditionsNode> {
    map(
        separated_list1(terminated(bool_op, whitespace), condition),
        |conditions| ConditionsNode { conditions },
    )(input)
}

fn if_statement(input: &str) -> IResult<IfNode> {
    context("if statement", |input| {
        let (input, _) = terminated(tag("if"), whitespace)(input)?;
        cut(|input| {
            let (input, _) = terminated(tag("("), whitespace)(input)?;
            let (input, conditions) = conditions(input)?;
            let (input, _) = terminated(tag(")"), whitespace)(input)?;
            let (input, body) = opt(map(
                tuple((
                    terminated(tag("{"), whitespace),
                    many0(if_statement),
                    terminated(tag("}"), whitespace),
                )),
                |(_, body, _)| body,
            ))(input)?;
            let (input, else_if_statements) = opt(many1(else_if_statement))(input)?;
            let (input, else_statement) = opt(else_statement)(input)?;

            let if_node = IfNode {
                body,
                conditions,
                else_if: else_if_statements,
                else_node: else_statement,
            };

            Ok((input, if_node))
        })(input)
    })(input)
}

fn else_if_statement(input: &str) -> IResult<ElseIfNode> {
    context("else if statement", |input| {
        let (input, _) = terminated(tag("else"), whitespace)(input)?;
        let (input, _) = terminated(tag("if"), whitespace)(input)?;
        let (input, _) = terminated(tag("("), whitespace)(input)?;
        let (input, conditions) = conditions(input)?;
        let (input, _) = terminated(tag(")"), whitespace)(input)?;
        let (input, body) = opt(map(
            tuple((
                terminated(tag("{"), whitespace),
                many0(if_statement),
                terminated(tag("}"), whitespace),
            )),
            |(_, body, _)| body,
        ))(input)?;

        let else_if_node = ElseIfNode { conditions, body };

        Ok((input, else_if_node))
    })(input)
}

fn else_statement(input: &str) -> IResult<ElseNode> {
    context("else statement", |input| {
        let (input, _) = terminated(tag("else"), whitespace)(input)?;
        let (input, _) = terminated(tag("{"), whitespace)(input)?;
        let (input, if_statements) = many0(if_statement)(input)?;
        let (input, _) = terminated(tag("}"), whitespace)(input)?;

        let else_node = ElseNode {
            body: if_statements,
        };

        Ok((input, else_node))
    })(input)
}

fn parse_float_type(input: &str) -> IResult<Type> {
    let (input, _) = terminated(tag("num"), whitespace)(input)?;
    let (input, _) = terminated(tag("("), whitespace)(input)?;
    cut(|input| {
        let (input, precision) = terminated(float, whitespace)(input)?;
        let (input, _) = terminated(tag(")"), whitespace)(input)?;

        Ok((input, Type::Float { precision }))
    })(input)
}

fn parse_bool_type(input: &str) -> IResult<Type> {
    let (input, _) = terminated(tag("bool"), whitespace)(input)?;

    Ok((input, Type::Bool))
}

fn parse_int_type(input: &str) -> IResult<Type> {
    let (input, _) = terminated(tag("int"), whitespace)(input)?;

    Ok((input, Type::Integer))
}

fn parse_simple_num_type(input: &str) -> IResult<Type> {
    let (input, _) = terminated(tag("num"), whitespace)(input)?;

    // TODO: This should be a default num precision somewhere
    Ok((input, Type::Float { precision: 0.01 }))
}

fn parse_type(input: &str) -> IResult<Type> {
    context(
        "type",
        alt((
            parse_bool_type,
            parse_int_type,
            complete(parse_float_type),
            parse_simple_num_type,
        )),
    )(input)
}

fn var_declaration(input: &str) -> IResult<VarNode> {
    context("var declaration", |input| {
        let (input, _) = terminated(tag("var"), whitespace)(input)?;
        cut(|input| {
            let (input, var_name) = terminated(var_name, whitespace)(input)?;
            let (input, _) = terminated(tag(":"), whitespace)(input)?;
            let (input, var_type) = terminated(parse_type, whitespace)(input)?;

            Ok((input, VarNode { var_name, var_type }))
        })(input)
    })(input)
}

fn feature(input: &str) -> IResult<FeatureNode> {
    enum VarOrIf<'a> {
        Var(VarNode<'a>),
        If(IfNode<'a>),
    }

    fn var_or_if(input: &str) -> IResult<VarOrIf> {
        alt((
            map(var_declaration, VarOrIf::Var),
            map(if_statement, VarOrIf::If),
        ))(input)
    }

    let (input, _) = terminated(tag("["), whitespace)(input)?;
    let (input, nodes) = cut(many1(var_or_if))(input)?;
    let (input, _) = cut(terminated(tag("]"), whitespace))(input)?;

    let (variables, if_statements) = nodes.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut variables, mut if_statements), x| {
            match x {
                VarOrIf::Var(var_node) => variables.push(var_node),
                VarOrIf::If(if_node) => if_statements.push(if_node),
            }
            (variables, if_statements)
        },
    );

    Ok((
        input,
        FeatureNode {
            variables,
            if_statements,
        },
    ))
}

fn root(input: &str) -> IResult<RootNode> {
    let (input, _) = whitespace(input)?;
    // TODO: top level could be a  list fo features not delimited by []
    let (input, features) = many0(terminated(feature, whitespace))(input)?;
    let (input, _) = eof(input)?;

    Ok((input, RootNode { features }))
}

pub fn parse_gpt_to_features(input: &str) -> IResult<Vec<Vec<NTupleInput>>> {
    let (input, ast) = root(input)?;
    let ir_features = ast_to_ir::convert_ast_to_ir(&ast);
    let ntuples_for_features = ir_features.iter().map(ir_to_ntuple::ir_to_ntuple).collect();

    Ok((input, ntuples_for_features))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace("     "), Ok(("", ())));
        assert_eq!(whitespace("     asd "), Ok(("asd ", ())));
        assert_eq!(whitespace("qwe"), Ok(("qwe", ())));
        assert_eq!(whitespace(""), Ok(("", ())));
        assert_eq!(whitespace("\n\t\n\t\t  \t  \n"), Ok(("", ())));
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
        assert_eq!(var_name("second_hand_price"), Ok(("", "second_hand_price")));
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
                    binary_op: BinaryOp::LessThanEqualTo
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
                    binary_op: BinaryOp::GreaterThan
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

        assert_eq!(
            conditions("true = x"),
            Ok((
                "",
                ConditionsNode {
                    conditions: vec![x_eq_true.clone()]
                }
            ))
        );
        assert_eq!(
            conditions("true = x && 0 > y"),
            Ok((
                "",
                ConditionsNode {
                    conditions: vec![x_eq_true.clone(), y_greater_0.clone()]
                }
            ))
        );
        assert_eq!(
            conditions("true = x && 0 > y && 0 > y    asd"),
            Ok((
                "asd",
                ConditionsNode {
                    conditions: vec![x_eq_true.clone(), y_greater_0.clone(), y_greater_0.clone()]
                }
            ))
        );
        assert!(conditions("").is_err());
        assert!(conditions("true = x &&").is_err());
    }

    #[test]
    fn test_if_statement() {
        assert_eq!(
            if_statement(
                "
            if (x >= 5 && y in (0, 10)) {
                if (x = true)
            } else if (x < 4 && y > 6)
            else {
                if (x != false)
            }   qwe
        "
                .trim()
            ),
            Ok((
                "qwe",
                IfNode {
                    conditions: conditions("x >= 5 && y in (0, 10)").unwrap().1,
                    body: Some(vec![IfNode {
                        conditions: conditions("x = true").unwrap().1,
                        body: None,
                        else_if: None,
                        else_node: None
                    }]),
                    else_if: Some(vec![ElseIfNode {
                        conditions: conditions("x < 4 && y > 6").unwrap().1,
                        body: None
                    }]),
                    else_node: Some(ElseNode {
                        body: vec![IfNode {
                            conditions: conditions("x != false").unwrap().1,
                            body: None,
                            else_if: None,
                            else_node: None
                        }]
                    })
                }
            ))
        );
        // TODO
    }

    // #[test]
    fn test_else_if_statement() {
        // TODO
        todo!()
    }

    // #[test]
    fn test_else_statement() {
        // TODO
        todo!()
    }

    // TODO: Make this test pass
    // #[test]
    fn test_feature() {
        let input = r#"
        [
            var VIP: bool 
            var price: num
            var second_hand_price: num
          
            if(VIP = true && price < 50 && price != 0) {
              if(price < 20 && second_hand_price > 60)
              if(price != 50)
            }
            else if(second_hand_price > 60)
          
            if(price > 30 && second_hand_price > 60)
          
            if(VIP = true) {
              if(second_hand_price = 2)
              if(second_hand_price = 3)
            }
          
            if(second_hand_price >= 50) {
              if(price < 5)
            }
            else if(10 < second_hand_price)
          
            if(price in [0,10] && price not in (9,100])
          
            if(VIP = true && price < 10) {
              if(second_hand_price = 2)
              if(second_hand_price = 3)
            }
            if(VIP = true) {
              if(second_hand_price = 2)
              if(second_hand_price = 3)
            }
            if(price < 10) {
              if(second_hand_price = 2)
              if(second_hand_price = 3)
            }
          
          
            if(price > 10) {
              if(price < 100) {
                if(price in [20,30])
              }
            }
          
            if(price in (-Inf,0) && price in (0,10])
          ]
        "#;

        assert_eq!(
            feature(input.trim()),
            Ok((
                "",
                FeatureNode {
                    variables: Vec::new(),
                    if_statements: Vec::new()
                }
            ))
        );
    }
}
