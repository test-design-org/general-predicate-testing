use nom::{
    branch::alt,
    combinator::{cut, map, value},
    error::context,
    sequence::{terminated, tuple},
};

use super::{
    ast::{
        BinaryCondition, BoolCondition, BoolOp, Condition, ConditionsNode, ConstantPosition,
        IntervalCondition,
    },
    interval,
    primitives::{binary_op, boolean, eq_op, interval_op, number, var_name},
    utils::{parenthesized, token, token_lit},
    IResult,
};

fn condition_bool_lhs(input: &str) -> IResult<Condition> {
    let (input, constant) = token(boolean)(input)?;
    let (input, eq_op) = token(eq_op)(input)?;
    let (input, var_name) = token(var_name)(input)?;

    Ok((
        input,
        Condition::Bool(BoolCondition {
            var_name,
            constant,
            eq_op,
        }),
    ))
}

fn condition_bool_rhs(input: &str) -> IResult<Condition> {
    let (input, var_name) = token(var_name)(input)?;
    let (input, eq_op) = token(eq_op)(input)?;
    let (input, constant) = token(boolean)(input)?;

    Ok((
        input,
        Condition::Bool(BoolCondition {
            var_name,
            constant,
            eq_op,
        }),
    ))
}

fn condition_binary_lhs(input: &str) -> IResult<Condition> {
    map(
        tuple((token(number), token(binary_op), token(var_name))),
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
        tuple((token(var_name), token(binary_op), token(number))),
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
        tuple((token(var_name), token(interval_op), token(interval))),
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

fn raw_expression(input: &str) -> IResult<ConditionsNode> {
    let (input, condition) = token(condition)(input)?;

    Ok((input, ConditionsNode::Expression(condition)))
}

fn negated(input: &str) -> IResult<ConditionsNode> {
    let (input, _) = token_lit("!")(input)?;
    let (input, node) = parenthesized(conditions)(input)?;

    Ok((input, ConditionsNode::Negated(Box::new(node))))
}

fn expression(input: &str) -> IResult<ConditionsNode> {
    context(
        "expression",
        alt((negated, parenthesized(conditions), cut(raw_expression))),
    )(input)
}

fn and_condition(input: &str) -> IResult<ConditionsNode> {
    alt((
        |input| {
            let (input, left) = expression(input)?;
            let (input, op) = value(BoolOp::And, token_lit("&&"))(input)?;
            let (input, right) = and_condition(input)?;

            Ok((
                input,
                ConditionsNode::Group {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator: op,
                },
            ))
        },
        expression,
    ))(input)
}

fn or_condition(input: &str) -> IResult<ConditionsNode> {
    alt((
        |input| {
            let (input, left) = and_condition(input)?;
            let (input, op) = value(BoolOp::Or, token_lit("||"))(input)?;
            let (input, right) = or_condition(input)?;

            Ok((
                input,
                ConditionsNode::Group {
                    left: Box::new(left),
                    right: Box::new(right),
                    operator: op,
                },
            ))
        },
        and_condition,
        expression,
    ))(input)
}

pub fn conditions(input: &str) -> IResult<ConditionsNode> {
    context("conditions", alt((or_condition, and_condition, expression)))(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        interval::{Boundary, MultiInterval},
        parser::ast::{BinaryOp, EqOp, IntervalOp},
    };

    #[test]
    fn test_condition_bool_lhs() {
        assert_eq!(
            condition_bool_lhs("true == x"),
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
            condition_bool_lhs("false==foo"),
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
            condition_bool_lhs("true   ==   y) asd"),
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
        assert!(condition_bool_lhs("false == false").is_err());
        assert!(condition_bool_lhs("true == true").is_err());
        assert!(condition_bool_lhs("x == true").is_err());
        assert!(condition_bool_lhs("false ==").is_err());
    }

    #[test]
    fn test_condition_bool_rhs() {
        assert_eq!(
            condition_bool_rhs("x == true"),
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
            condition_bool_rhs("foo==false"),
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
            condition_bool_rhs("y   ==   true) asd"),
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
        assert!(condition_bool_rhs("false == false").is_err());
        assert!(condition_bool_rhs("true == true").is_err());
        assert!(condition_bool_rhs("true == x").is_err());
        assert!(condition_bool_rhs("x ==").is_err());
    }

    #[test]
    fn test_condition_binary_lhs() {
        assert_eq!(
            condition_binary_lhs("10.32 == x"),
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
        assert!(condition_binary_lhs("x == 123.0").is_err());
        assert!(condition_binary_lhs("true == x").is_err());
        assert!(condition_binary_lhs("123 == 123").is_err());
        assert!(condition_binary_lhs("123.0 >=").is_err());
    }

    #[test]
    fn test_condition_binary_rhs() {
        assert_eq!(
            condition_binary_rhs("x == 10.32"),
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
        assert!(condition_binary_rhs("123.0 == x").is_err());
        assert!(condition_binary_rhs("x == true").is_err());
        assert!(condition_binary_rhs("123 == 123").is_err());
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
            condition("true == x"),
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
            condition("x == true"),
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
            condition("10.32 == x"),
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
            condition("x == 10.32"),
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
            conditions("true == x"),
            Ok(("", ConditionsNode::Expression(x_eq_true.clone())))
        );
        assert_eq!(
            conditions("true == x && 0 > y"),
            Ok((
                "",
                ConditionsNode::Group {
                    left: Box::new(ConditionsNode::Expression(x_eq_true.clone())),
                    right: Box::new(ConditionsNode::Expression(y_greater_0.clone())),
                    operator: BoolOp::And,
                }
            ))
        );
        assert_eq!(
            conditions("true == x && 0 > y && 0 > y    asd"),
            Ok((
                "asd",
                ConditionsNode::Group {
                    left: Box::new(ConditionsNode::Expression(x_eq_true.clone())),
                    right: Box::new(ConditionsNode::Group {
                        left: Box::new(ConditionsNode::Expression(y_greater_0.clone())),
                        right: Box::new(ConditionsNode::Expression(y_greater_0.clone())),
                        operator: BoolOp::And,
                    }),
                    operator: BoolOp::And,
                }
            ))
        );
        assert_eq!(
            conditions("(true == x && 0 > y) && 0 > y    asd"),
            Ok((
                "asd",
                ConditionsNode::Group {
                    left: Box::new(ConditionsNode::Group {
                        left: Box::new(ConditionsNode::Expression(x_eq_true.clone())),
                        right: Box::new(ConditionsNode::Expression(y_greater_0.clone())),
                        operator: BoolOp::And,
                    }),
                    right: Box::new(ConditionsNode::Expression(y_greater_0.clone())),
                    operator: BoolOp::And,
                }
            ))
        );
        assert_eq!(
            conditions("true == x || 0 > y && 0 > y    asd"),
            Ok((
                "asd",
                ConditionsNode::Group {
                    left: Box::new(ConditionsNode::Expression(x_eq_true.clone())),
                    right: Box::new(ConditionsNode::Group {
                        left: Box::new(ConditionsNode::Expression(y_greater_0.clone())),
                        right: Box::new(ConditionsNode::Expression(y_greater_0.clone())),
                        operator: BoolOp::And,
                    }),
                    operator: BoolOp::Or,
                }
            ))
        );
        // TODO: Add a bunch more tests for testing good precedence detection and stuff
        assert!(conditions("").is_err());
        assert!(conditions("true == x &&").is_err());
    }
}
