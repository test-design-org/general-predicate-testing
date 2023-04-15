use std::fmt;

use itertools::Itertools;

use crate::{
    interval::MultiInterval,
    parser::ast::{BoolOp, Type},
};

pub mod ast_to_ir;
pub mod ir_to_ntuple;

#[derive(PartialEq, Clone, Debug)]
pub struct Variable {
    pub var_name: String,
    pub var_type: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoolCondition {
    pub var_name: String,
    pub should_equal_to: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntervalCondition {
    pub var_name: String,
    pub interval: MultiInterval,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    Bool(BoolCondition),
    Interval(IntervalCondition),
}

impl Condition {
    pub fn get_variable(&self) -> &str {
        match self {
            Self::Bool(cond) => cond.var_name.as_str(),
            Self::Interval(cond) => cond.var_name.as_str(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Predicate {
    Negated(Box<Predicate>),
    Expression(Condition),
    Group {
        left: Box<Predicate>,
        right: Box<Predicate>,
        operator: BoolOp,
    },
}

impl Predicate {
    pub fn negated(&self) -> Predicate {
        match self {
            Self::Negated(pred) => pred.as_ref().clone(),
            Self::Expression(cond) => match cond {
                Condition::Bool(cond) => Self::Expression(Condition::Bool(BoolCondition {
                    var_name: cond.var_name.clone(),
                    should_equal_to: !cond.should_equal_to,
                })),
                Condition::Interval(cond) => {
                    Self::Expression(Condition::Interval(IntervalCondition {
                        var_name: cond.var_name.clone(),
                        interval: cond.interval.inverse(),
                    }))
                }
            },
            Self::Group {
                left,
                right,
                operator,
            } => Self::Group {
                left: Box::new(left.as_ref().negated()),
                right: Box::new(right.as_ref().negated()),
                operator: match operator {
                    BoolOp::And => BoolOp::Or,
                    BoolOp::Or => BoolOp::And,
                },
            },
        }
    }

    fn to_ands(&self) -> Vec<Predicate> {
        match self {
            Self::Negated(pred) => pred.negated().to_ands(),
            Self::Expression(cond) => vec![Self::Expression(cond.clone())],
            Self::Group {
                left,
                right,
                operator,
            } => match operator {
                BoolOp::And => {
                    let left_ands = left.as_ref().to_ands();
                    let right_ands = right.as_ref().to_ands();

                    left_ands
                        .into_iter()
                        .cartesian_product(right_ands.into_iter())
                        .map(|(left, right)| Predicate::Group {
                            left: Box::new(left),
                            right: Box::new(right),
                            operator: BoolOp::And,
                        })
                        .collect()
                }
                BoolOp::Or => {
                    let left_ands = left.as_ref().to_ands();
                    let left_negated = left.as_ref().negated();
                    let right_ands = right.as_ref().to_ands();

                    let mut left_negated_and_right_ands = right_ands
                        .into_iter()
                        .map(|right| Predicate::Group {
                            left: Box::new(left_negated.clone()),
                            right: Box::new(right),
                            operator: BoolOp::And,
                        })
                        .collect();

                    let mut result = left_ands;
                    result.append(&mut left_negated_and_right_ands);

                    result
                }
            },
        }
    }

    fn one_pred_foo(&self) -> Vec<Vec<Condition>> {
        match self {
            Self::Negated(_) => {
                panic!("This should not happen, in conjunction_of_conditions after we've called self.to_ands() there is a Negated condition. to_ands() should have taken care of that");
            }
            Self::Expression(cond) => vec![vec![cond.clone()]],
            Self::Group {
                left,
                right,
                operator: BoolOp::And,
            } => {
                let left_conds = left.as_ref().conjunction_of_conditions();
                let right_conds = right.as_ref().conjunction_of_conditions();

                let mut result = Vec::new();
                for left in left_conds {
                    for right in right_conds.clone() {
                        let mut new = left.clone();
                        new.append(&mut right.clone());
                        result.push(new);
                    }
                }

                result
            }
            Self::Group {
                operator: BoolOp::Or,
                ..
            } => {
                panic!("This should not happen, in conjunction_of_conditions after we've called self.to_ands() there is a BoolOp::Or");
            }
        }
    }

    pub fn conjunction_of_conditions(&self) -> Vec<Vec<Condition>> {
        self.to_ands()
            .iter()
            .flat_map(|p| p.one_pred_foo())
            .collect()
    }
}

pub struct Feature {
    pub variables: Vec<Variable>,
    pub predicates: Vec<Predicate>,
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negated(pred) => write!(f, "!({})", pred),
            Self::Expression(cond) => match cond {
                Condition::Bool(cond) => write!(f, "{} == {}", cond.var_name, cond.should_equal_to),
                Condition::Interval(cond) => write!(f, "{} in {}", cond.var_name, cond.interval),
            },
            Self::Group {
                left,
                right,
                operator,
            } => write!(
                f,
                "({} {} {})",
                left.as_ref(),
                match operator {
                    BoolOp::And => "&&",
                    BoolOp::Or => "||",
                },
                right.as_ref()
            ),
        }
    }
}

impl fmt::Debug for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::{BoolCondition, Condition, IntervalCondition, Predicate};
    use crate::{interval::test::multiint, parser::ast::BoolOp};

    fn cond<'a>(var_name: &'a str, interval: &'a str) -> Condition {
        Condition::Interval(IntervalCondition {
            var_name: var_name.to_owned(),
            interval: multiint(interval),
        })
    }

    fn expr<'a>(var_name: &'a str, interval: &'a str) -> Box<Predicate> {
        Box::new(Predicate::Expression(cond(var_name, interval)))
    }

    fn and(left: Box<Predicate>, right: Box<Predicate>) -> Box<Predicate> {
        Box::new(Predicate::Group {
            left,
            right,
            operator: BoolOp::And,
        })
    }

    #[test]
    fn test_to_ands() {
        let predicate = Predicate::Group {
            left: Box::new(Predicate::Group {
                left: expr("x", "[0,0]"),
                right: expr("y", "[0,0]"),
                operator: BoolOp::Or,
            }),
            right: Box::new(Predicate::Group {
                left: expr("x", "[1,1]"),
                right: expr("y", "[1,1]"),
                operator: BoolOp::Or,
            }),
            operator: BoolOp::Or,
        };

        println!("Input predicate:\n{:#?}\n", predicate);
        println!("Conjuntive form:\n{:#?}", predicate.to_ands());

        let expected = vec![
            expr("x", "[0,0]").as_ref().to_owned(),
            Predicate::Group {
                left: expr("x", "(-Inf, 0) (0, Inf)"),
                right: expr("y", "[0,0]"),
                operator: BoolOp::And,
            },
            Predicate::Group {
                left: and(
                    expr("x", "(-Inf, 0) (0, Inf)"),
                    expr("y", "(-Inf, 0) (0, Inf)"),
                ),
                right: expr("x", "[1,1]"),
                operator: BoolOp::And,
            },
            Predicate::Group {
                left: and(
                    expr("x", "(-Inf, 0) (0, Inf)"),
                    expr("y", "(-Inf, 0) (0, Inf)"),
                ),
                right: and(expr("x", "(-Inf, 1) (1, Inf)"), expr("y", "[1,1]")),
                operator: BoolOp::And,
            },
        ];

        let actual = predicate.to_ands();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_conjunction_of_conditions() {
        let predicate = Predicate::Group {
            left: Box::new(Predicate::Group {
                left: expr("x", "[0,0]"),
                right: expr("y", "[0,0]"),
                operator: BoolOp::Or,
            }),
            right: Box::new(Predicate::Group {
                left: expr("x", "[1,1]"),
                right: expr("y", "[1,1]"),
                operator: BoolOp::Or,
            }),
            operator: BoolOp::Or,
        };

        // println!("Input predicate:\n{:#?}\n", predicate);
        // println!(
        //     "Conjuntive form:\n{:#?}",
        //     predicate.conjunction_of_conditions()
        // );

        let expected = vec![
            vec![cond("x", "[0,0]")],
            vec![cond("x", "(-Inf, 0) (0, Inf)"), cond("y", "[0,0]")],
            vec![
                cond("x", "(-Inf, 0) (0, Inf)"),
                cond("y", "(-Inf, 0) (0, Inf)"),
                cond("x", "[1,1]"),
            ],
            vec![
                cond("x", "(-Inf, 0) (0, Inf)"),
                cond("y", "(-Inf, 0) (0, Inf)"),
                cond("x", "(-Inf, 1) (1, Inf)"),
                cond("y", "[1,1]"),
            ],
        ];

        let actual = predicate.conjunction_of_conditions();

        assert_eq!(actual, expected);
    }
}
