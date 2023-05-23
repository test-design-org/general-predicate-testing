use std::fmt;

use itertools::Itertools;

use crate::{
    interval::MultiInterval,
    parser::ast::{BoolOp, Type},
    util::{ContinousSublistsFromFirst, UniquesVec},
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

    fn negated(&self) -> Self {
        match self {
            Self::Bool(cond) => Self::Bool(BoolCondition {
                var_name: cond.var_name.clone(),
                should_equal_to: !cond.should_equal_to,
            }),
            Self::Interval(cond) => Self::Interval(IntervalCondition {
                var_name: cond.var_name.clone(),
                interval: cond.interval.complement(),
            }),
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Bool(BoolCondition {
                var_name,
                should_equal_to,
            }) => write!(f, "{var_name} == {should_equal_to}"),
            Condition::Interval(IntervalCondition { var_name, interval }) => {
                write!(f, "{var_name} in {interval}")
            }
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
            Self::Expression(cond) => Self::Expression(cond.negated()),
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

    pub fn reduce(&self) -> ReducedPredicate {
        match self {
            Predicate::Negated(x) => x.as_ref().negated().reduce(),
            Predicate::Expression(x) => ReducedPredicate::Expression(x.clone()),
            Predicate::Group {
                left,
                right,
                operator: BoolOp::And,
            } => {
                let mut ands = Vec::new();
                let mut ors = Vec::new();

                let mut handle_reduced_sub_predicate = |reduced_sub_predicate: ReducedPredicate| {
                    match reduced_sub_predicate {
                        ReducedPredicate::Expression(x) => ands.push(x),
                        ReducedPredicate::And(Ands {
                            conjugated_conditions,
                            sub_ors,
                        }) => {
                            ands.append(&mut conjugated_conditions.clone());
                            ors.append(&mut sub_ors.clone());
                        }
                        ReducedPredicate::Or(or) => ors.push(or),
                    };
                };

                handle_reduced_sub_predicate(left.as_ref().reduce());
                handle_reduced_sub_predicate(right.as_ref().reduce());

                ReducedPredicate::And(Ands {
                    conjugated_conditions: ands,
                    sub_ors: ors,
                })
            }
            Predicate::Group {
                left,
                right,
                operator: BoolOp::Or,
            } => {
                let mut ors = Vec::new();
                let mut ands = Vec::new();
                let mut handle_reduced_sub_predicate = |reduced_sub_predicate: ReducedPredicate| {
                    match reduced_sub_predicate {
                        ReducedPredicate::Expression(x) => ors.push(x),
                        ReducedPredicate::Or(Ors {
                            disjuncted_conditions,
                            sub_ands,
                        }) => {
                            ors.append(&mut disjuncted_conditions.clone());
                            ands.append(&mut sub_ands.clone());
                        }
                        ReducedPredicate::And(and) => ands.push(and),
                    };
                };

                handle_reduced_sub_predicate(left.as_ref().reduce());
                handle_reduced_sub_predicate(right.as_ref().reduce());

                ReducedPredicate::Or(Ors {
                    disjuncted_conditions: ors,
                    sub_ands: ands,
                })
            }
        }
    }

    pub fn conjunction_of_conditions(&self) -> Vec<Vec<Condition>> {
        self.reduce().to_ands()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ands {
    conjugated_conditions: Vec<Condition>,
    sub_ors: Vec<Ors>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ors {
    disjuncted_conditions: Vec<Condition>,
    sub_ands: Vec<Ands>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReducedPredicate {
    Expression(Condition),
    And(Ands),
    Or(Ors),
}

impl ReducedPredicate {
    fn to_ands(&self) -> Vec<Vec<Condition>> {
        match self {
            Self::Expression(cond) => vec![vec![cond.clone()]],
            Self::And(Ands {
                conjugated_conditions,
                sub_ors,
            }) => {
                if sub_ors.is_empty() {
                    vec![conjugated_conditions.clone()]
                } else {
                    let mut result = Vec::new();

                    let ands_from_ors: Vec<Vec<Vec<Condition>>> = sub_ors
                        .iter()
                        .map(|ors| ReducedPredicate::Or(ors.clone()).to_ands())
                        .collect();

                    for (a, b) in ands_from_ors.iter().tuple_combinations() {
                        let mut asd = a
                            .iter()
                            .cartesian_product(b.iter())
                            .map(|(x, y)| {
                                let mut z = x.clone();
                                z.append(&mut y.clone());
                                z
                            })
                            .collect::<Vec<Vec<Condition>>>();

                        for x in &mut asd {
                            let mut base = conjugated_conditions.clone();
                            base.append(x);
                            result.push(base);
                        }
                    }

                    result
                }
            }
            Self::Or(Ors {
                disjuncted_conditions,
                sub_ands,
            }) => {
                let all_possible_anded_form_of_the_or = disjuncted_conditions
                    .iter()
                    .permutations(disjuncted_conditions.len())
                    // .take(1) // TODO: REMOVEEE. What this does is basically ignore all the possible permutations of the or conditions
                    .flat_map(|one_permuation| {
                        one_permuation
                            .continous_sublists_from_first()
                            .iter()
                            .map(|xs| {
                                let (last, ys) =
                                    xs.split_last().expect("We've filtered out empty vecs");

                                let mut ys = ys.to_vec();
                                // We're sorting it, so `!x && !y` and `!y && !x` can be deduplicated with the unique call
                                ys.sort_by_key(|cond| cond.get_variable());

                                let mut zs = ys
                                    .iter()
                                    .map(|cond| cond.negated())
                                    .collect::<Vec<Condition>>();

                                zs.push((*last).clone());

                                zs
                            })
                            .collect::<Vec<Vec<Condition>>>()
                    })
                    .collect::<Vec<Vec<Condition>>>()
                    .uniques(); // TODO: Itertools::unique would be betetr, but that requires Eq and Hash

                if sub_ands.is_empty() {
                    all_possible_anded_form_of_the_or
                } else {
                    let ands_from_ands: Vec<Vec<Vec<Condition>>> = sub_ands
                        .iter()
                        .map(|and| ReducedPredicate::And(and.clone()).to_ands())
                        .collect();

                    all_possible_anded_form_of_the_or
                        .iter()
                        .flat_map(|conjugated_conditions| {
                            let mut result = Vec::new();
                            for (a, b) in ands_from_ands.iter().tuple_combinations() {
                                let mut asd = a
                                    .iter()
                                    .cartesian_product(b.iter())
                                    .map(|(x, y)| {
                                        let mut z = x.clone();
                                        z.append(&mut y.clone());
                                        z
                                    })
                                    .collect::<Vec<Vec<Condition>>>();

                                for x in &mut asd {
                                    let mut base = conjugated_conditions.clone();
                                    base.append(x);
                                    result.push(base);
                                }
                            }

                            result
                        })
                        .collect()
                }
            }
        }
    }

    // fn one_pred_foo(and: Vec<Condition>) -> Vec<Vec<Condition>> {
    //     match self {
    //         Self::Expression(cond) => vec![vec![cond.clone()]],
    //         Self::Group(PredicateGroup {
    //             left,
    //             right,
    //             operator: BoolOp::And,
    //         }) => {
    //             let left_conds = left.as_ref().conjunction_of_conditions();
    //             let right_conds = right.as_ref().conjunction_of_conditions();

    //             let mut result = Vec::new();
    //             for left in left_conds {
    //                 for right in right_conds.clone() {
    //                     let mut new = left.clone();
    //                     new.append(&mut right.clone());
    //                     result.push(new);
    //                 }
    //             }

    //             result
    //         }
    //     }
    // }

    // pub fn conjunction_of_conditions(&self) -> Vec<Vec<Condition>> {
    //     self.to_ands()
    //         .iter()
    //         .flat_map(|p| p.one_pred_foo())
    //         .collect()
    // }
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
    use pretty_assertions::assert_eq;

    use super::{Condition, IntervalCondition, Predicate};
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

    // #[test]
    // fn test_to_ands() {
    //     let predicate = Predicate::Group {
    //         left: Box::new(Predicate::Group {
    //             left: expr("x", "[0,0]"),
    //             right: expr("y", "[0,0]"),
    //             operator: BoolOp::Or,
    //         }),
    //         right: Box::new(Predicate::Group {
    //             left: expr("x", "[1,1]"),
    //             right: expr("y", "[1,1]"),
    //             operator: BoolOp::Or,
    //         }),
    //         operator: BoolOp::Or,
    //     };

    //     println!("Input predicate:\n{:#?}\n", predicate);
    //     println!("Conjuntive form:\n{:#?}", predicate.to_ands());

    //     let expected = vec![
    //         expr("x", "[0,0]").as_ref().to_owned(),
    //         Predicate::Group {
    //             left: expr("x", "(-Inf, 0) (0, Inf)"),
    //             right: expr("y", "[0,0]"),
    //             operator: BoolOp::And,
    //         },
    //         Predicate::Group {
    //             left: and(
    //                 expr("x", "(-Inf, 0) (0, Inf)"),
    //                 expr("y", "(-Inf, 0) (0, Inf)"),
    //             ),
    //             right: expr("x", "[1,1]"),
    //             operator: BoolOp::And,
    //         },
    //         Predicate::Group {
    //             left: and(
    //                 expr("x", "(-Inf, 0) (0, Inf)"),
    //                 expr("y", "(-Inf, 0) (0, Inf)"),
    //             ),
    //             right: and(expr("x", "(-Inf, 1) (1, Inf)"), expr("y", "[1,1]")),
    //             operator: BoolOp::And,
    //         },
    //     ];

    //     let actual = predicate.to_ands();

    //     assert_eq!(actual, expected);
    // }

    // TODO: This test case has too much elements after the variable order change, it should be revised
    // #[test]
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
