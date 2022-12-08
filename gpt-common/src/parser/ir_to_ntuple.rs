use crate::dto::{BoolDTO, BoolExpression, Expression, Input, IntervalDTO, NTuple};

use super::{
    ast::Type,
    ir::{self, Feature},
};

fn convert_bool_dto(condition: ir::BoolCondition) -> BoolDTO {
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

fn convert_interval_dto(variable: ir::Variable, condition: ir::IntervalCondition) -> IntervalDTO {
    let precision = variable.var_type.get_precision().expect("Type error: when converting an interval dto in convert_interval_dto, the variable type doesn't have a precision!");

    IntervalDTO {
        expression: condition.expression,
        interval: condition.interval.DONOTUSE_get_interval(),
        precision,
        is_constant: false,
    }
}

fn convert_condition(variable: ir::Variable, condition: ir::Condition) -> Input {
    match condition {
        ir::Condition::Bool(cond) => Input::Bool(convert_bool_dto(cond)),
        ir::Condition::Interval(cond) => Input::Interval(convert_interval_dto(variable, cond)),
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

fn convert_predicate_to_ntuple(variables: &Vec<ir::Variable>, predicate: &ir::Predicate) -> NTuple {
    let inputs = variables
        .clone()
        .into_iter()
        .map(|variable| {
            predicate
                .clone()
                .into_iter()
                .find(|cond| cond.get_variable() == variable.var_name)
                .map(|cond| convert_condition(variable, cond))
                .unwrap_or(Input::MissingVariable)
        })
        .collect();

    NTuple { inputs }
}

pub fn ir_to_ntuple<'a>(
    Feature {
        variables,
        predicates,
    }: &Feature<'a>,
) -> Vec<NTuple> {
    predicates
        .clone()
        .iter()
        .map(|predicate| convert_predicate_to_ntuple(&variables, predicate))
        .collect()
}
