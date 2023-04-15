use std::collections::HashMap;

use super::{BoolCondition, Condition, Feature, IntervalCondition};
use crate::{
    dto::{BoolDTO, BoolExpression, Input, IntervalDTO, NTupleInput},
    interval::Intersectable,
    ir,
};

const fn convert_bool_dto(condition: &ir::BoolCondition) -> BoolDTO {
    let expression = if condition.should_equal_to == true {
        BoolExpression::IsTrue
    } else {
        BoolExpression::IsFalse
    };
    BoolDTO {
        expression,
        bool_val: condition.should_equal_to,
        is_constant: false,
    }
}

fn convert_interval_dto(variable: &ir::Variable, condition: &ir::IntervalCondition) -> IntervalDTO {
    let precision = variable.var_type.get_precision().expect("Type error: when converting an interval dto in convert_interval_dto, the variable type doesn't have a precision!");

    IntervalDTO {
        interval: condition.interval.clone(),
        precision,
        is_constant: false,
    }
}

fn convert_condition(variable: &ir::Variable, condition: ir::Condition) -> Input {
    match condition {
        ir::Condition::Bool(cond) => Input::Bool(convert_bool_dto(&cond)),
        ir::Condition::Interval(cond) => Input::Interval(convert_interval_dto(variable, &cond)),
    }
}

// fn sort_objects_into_tuple<TTupleElem, TObject, TCompare>(
//     tuple_format: Vec<TTupleElem>,
//     objects: &Vec<TObject>,
//     lens_tuple_elem: &mut dyn FnMut(&TTupleElem) -> TCompare,
//     lens_object: &mut dyn FnMut(&TObject) -> TCompare,
// ) -> Vec<(TTupleElem, Option<TObject>)>
// where
//     TObject: Copy,
//     TCompare: PartialEq,
// {
//     let objs = objects.clone();

//     tuple_format
//         .into_iter()
//         .map(|tuple_elem| {
//             let foo: Option<TObject> = objs
//                 .clone()
//                 .into_iter()
//                 .find(|x| lens_object(x) == lens_tuple_elem(&tuple_elem));

//             (tuple_elem, foo)
//         })
//         .collect()
// }

fn convert_predicate_to_ntuple(
    variables: &[ir::Variable],
    predicate: &ir::Predicate,
) -> Vec<NTupleInput> {
    predicate
        .conjunction_of_conditions()
        .into_iter()
        .filter_map(|conditions| {
            conditions
                .into_iter()
                .fold(
                    Some(HashMap::<String, Condition>::new()),
                    |ntuple, cond| {
                        ntuple.map(|mut ntuple| {
                            let var_name = cond.get_variable();
                            let to_insert = match (&cond, ntuple.get(var_name)) {
                                (x, None) => Some(x.clone()),
                                (
                                     Condition::Bool(BoolCondition {
                                        should_equal_to: old,
                                        ..
                                    }),
                                    Some(Condition::Bool(BoolCondition {
                                        should_equal_to: new,
                                        ..
                                    })),
                                ) => {
                                    if old != new {
                                        None
                                    } else {
                                        Some(Condition::Bool(BoolCondition {
                                            var_name: var_name.to_owned(),
                                            should_equal_to: *old,
                                        }))
                                    }
                                }
                                (
                                    Condition::Interval(IntervalCondition {
                                        interval: old, ..
                                    }),
                                    Some(Condition::Interval(IntervalCondition {
                                        interval: new,
                                        ..
                                    })),
                                ) => old.intersect(new).map(|intersection| {
                                    Condition::Interval(IntervalCondition {
                                        interval: intersection,
                                        var_name: var_name.to_owned(),
                                    })
                                }),
                                (x, y) => panic!("Mismatched types in predicate! Variable {var_name} has both a boolean and an interval condition! {x:#?} and {y:#?}")
                            };
                            if let Some(to_insert) = to_insert {
                                ntuple.insert(var_name.to_owned(), to_insert);
                            }
                            ntuple
                        })
                    },
                )
                .map(|x| x.into_iter().map(|(var_name, condition)| {
                    let variable = variables
                        .iter()
                        .find(|variable| var_name.as_str() == variable.var_name)
                        // TODO: This should be an actual error in a Result type
                        .unwrap_or_else(|| {
                            panic!("Undefined variable: {}", var_name)
                        });
                    (
                        var_name,
                        convert_condition(variable, condition),
                    )
                }))
                .map(|x| NTupleInput {inputs: x.collect() })
        })
        .collect()
}

pub fn ir_to_ntuple(
    Feature {
        variables,
        predicates,
    }: &Feature,
) -> Vec<NTupleInput> {
    predicates
        .clone()
        .iter()
        .flat_map(|predicate| convert_predicate_to_ntuple(variables, predicate))
        .collect()
}
