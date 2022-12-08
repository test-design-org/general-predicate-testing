use crate::{
    dto::Expression,
    interval::{Boundary, MultiInterval},
};

use super::{
    ast::{self, BinaryOp, ConstantPosition, EqOp, RootNode},
    ir::{self, IntervalCondition},
};

const fn map_binary_op_to_expression(binop: &BinaryOp) -> Expression {
    match binop {
        BinaryOp::LessThan => Expression::LessThan,
        BinaryOp::GreaterThan => Expression::GreaterThan,
        BinaryOp::LessThanEqualTo => Expression::LessThanOrEqualTo,
        BinaryOp::GreaterThanEqualTo => Expression::GreaterThanOrEqualTo,
        BinaryOp::Equal => Expression::EqualTo,
        BinaryOp::NotEqual => Expression::NotEqualTo,
    }
}

const fn resolve_bool_condition(eq_op: &EqOp, bool_val: bool) -> bool {
    /*
    x = true    ->  true
    x != true   ->  false
    x = false   ->  false
    x != false  ->  true
    */
    match eq_op {
        EqOp::Equal => bool_val,
        EqOp::NotEqual => !bool_val,
    }
}

fn binary_op_to_interval(binop: &BinaryOp, num: f32) -> MultiInterval {
    use Boundary::{Closed, Open};

    let (lo_boundary, lo, hi, hi_boundary) = match binop {
        BinaryOp::LessThan => (Open, f32::NEG_INFINITY, num, Open),
        BinaryOp::GreaterThan => (Open, num, f32::INFINITY, Open),
        BinaryOp::LessThanEqualTo => (Open, f32::NEG_INFINITY, num, Closed),
        BinaryOp::GreaterThanEqualTo => (Closed, num, f32::INFINITY, Open),
        BinaryOp::Equal => (Closed, num, num, Closed),

        // TODO: This should return (-Inf,num) (num, Inf) as a multi-interval
        BinaryOp::NotEqual => (Open, num, num, Open),
    };

    MultiInterval::new(lo_boundary, lo, hi, hi_boundary)
        .expect("in binary_op_to_interval we've checked that lo <= hi")
}

const fn convert_bool_condition<'a>(cond: &'a ast::BoolCondition) -> ir::Condition<'a> {
    let should_equal_to = resolve_bool_condition(&cond.eq_op, cond.constant);

    ir::Condition::Bool(ir::BoolCondition {
        var_name: cond.var_name,
        should_equal_to,
    })
}

fn convert_binary_condition<'a>(cond: &'a ast::BinaryCondition) -> ir::Condition<'a> {
    // createUnaryIntervalDTO expects the constant to be on the right, like: x < 0
    // If it was inputted on the left like 0 > x we should flip it to be x < 0
    let binary_op = if cond.constant_position == ConstantPosition::LeftHandSide {
        cond.binary_op.flip()
    } else {
        cond.binary_op.clone()
    };

    let expression = map_binary_op_to_expression(&binary_op);

    ir::Condition::Interval(IntervalCondition {
        var_name: cond.var_name,
        expression,
        interval: binary_op_to_interval(&binary_op, cond.constant),
    })
}

fn convert_interval_condition<'a>(cond: &'a ast::IntervalCondition) -> ir::Condition<'a> {
    ir::Condition::Interval(ir::IntervalCondition {
        var_name: cond.var_name,
        expression: Expression::Interval,
        interval: cond.interval.clone(),
    })
}

fn convert_condition_node<'a>(conditions_node: &'a ast::ConditionsNode) -> ir::Predicate<'a> {
    let conditions = conditions_node
        .conditions
        .iter()
        .map(|cond| match cond {
            ast::Condition::Bool(cond) => convert_bool_condition(cond),
            ast::Condition::Binary(cond) => convert_binary_condition(cond),
            ast::Condition::Interval(cond) => convert_interval_condition(cond),
        })
        .collect();

    conditions
}

fn traverse_if_node<'a>(if_node: &'a ast::IfNode) -> Vec<ir::Predicate<'a>> {
    if if_node.else_if.as_ref().map_or(false, |x| !x.is_empty()) {
        panic!("Else if not yet supported!")
    }

    if if_node.else_node.is_some() {
        panic!("Else not yet supported!")
    }

    let initial_conditions = convert_condition_node(&if_node.conditions);

    match &if_node.body {
        None => vec![initial_conditions],
        Some(body_if_nodes) => {
            let body_conditions = body_if_nodes.iter().flat_map(traverse_if_node);
            body_conditions
                .map(|body_condition| {
                    let conds = [initial_conditions.as_slice(), body_condition.as_slice()].concat();
                    conds
                })
                .collect()
        }
    }
}

const fn convert_variable<'a>(var_node: &'a ast::VarNode) -> ir::Variable<'a> {
    ir::Variable {
        var_name: var_node.var_name,
        var_type: var_node.var_type,
    }
}

fn traverse_feature_node<'a>(feature_node: &'a ast::FeatureNode) -> ir::Feature<'a> {
    let variables = feature_node
        .variables
        .iter()
        .map(convert_variable)
        .collect();

    let predicates = feature_node
        .if_statements
        .iter()
        .flat_map(traverse_if_node)
        .collect();

    ir::Feature {
        variables,
        predicates,
    }
}

pub fn convert_ast_to_ir<'a>(root: &'a RootNode<'a>) -> Vec<ir::Feature<'a>> {
    root.features.iter().map(traverse_feature_node).collect()
}
