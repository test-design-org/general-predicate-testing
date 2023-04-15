use super::{IntervalCondition, Predicate};
use crate::{
    interval::{Boundary, MultiInterval},
    ir,
    parser::ast::{self, BinaryOp, BoolOp, ConstantPosition, ElseNode, EqOp, IfNode, RootNode},
};

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

    match binop {
        BinaryOp::NotEqual => MultiInterval::new_closed_point(num).inverse(),
        x => {
            let (lo_boundary, lo, hi, hi_boundary) = match x {
                BinaryOp::LessThan => (Open, f32::NEG_INFINITY, num, Open),
                BinaryOp::GreaterThan => (Open, num, f32::INFINITY, Open),
                BinaryOp::LessThanEqualTo => (Open, f32::NEG_INFINITY, num, Closed),
                BinaryOp::GreaterThanEqualTo => (Closed, num, f32::INFINITY, Open),
                BinaryOp::Equal => (Closed, num, num, Closed),
                _ => unreachable!(),
            };

            MultiInterval::new(lo_boundary, lo, hi, hi_boundary)
                .expect("in binary_op_to_interval we've checked that lo <= hi")
        }
    }
}

fn convert_bool_condition(cond: &ast::BoolCondition) -> ir::Condition {
    let should_equal_to = resolve_bool_condition(&cond.eq_op, cond.constant);

    ir::Condition::Bool(ir::BoolCondition {
        var_name: cond.var_name.to_owned(),
        should_equal_to,
    })
}

fn convert_binary_condition(cond: &ast::BinaryCondition) -> ir::Condition {
    // createUnaryIntervalDTO expects the constant to be on the right, like: x < 0
    // If it was inputted on the left like 0 > x we should flip it to be x < 0
    let binary_op = if cond.constant_position == ConstantPosition::LeftHandSide {
        cond.binary_op.flip()
    } else {
        cond.binary_op.clone()
    };

    ir::Condition::Interval(IntervalCondition {
        var_name: cond.var_name.to_owned(),
        interval: binary_op_to_interval(&binary_op, cond.constant),
    })
}

fn convert_interval_condition(cond: &ast::IntervalCondition) -> ir::Condition {
    ir::Condition::Interval(ir::IntervalCondition {
        var_name: cond.var_name.to_owned(),
        interval: match cond.interval_op {
            ast::IntervalOp::In => cond.interval.clone(),
            ast::IntervalOp::NotIn => cond.interval.inverse(),
        },
    })
}

fn convert_condition_node(conditions_node: &ast::ConditionsNode) -> ir::Predicate {
    match conditions_node {
        ast::ConditionsNode::Negated(cond) => {
            ir::Predicate::Negated(Box::new(convert_condition_node(cond)))
        }
        ast::ConditionsNode::Expression(cond) => ir::Predicate::Expression(match cond {
            ast::Condition::Bool(cond) => convert_bool_condition(cond),
            ast::Condition::Binary(cond) => convert_binary_condition(cond),
            ast::Condition::Interval(cond) => convert_interval_condition(cond),
        }),
        ast::ConditionsNode::Group {
            operator,
            left,
            right,
        } => ir::Predicate::Group {
            left: Box::new(convert_condition_node(left)),
            right: Box::new(convert_condition_node(right)),
            operator: *operator,
        },
    }
}

fn traverse_body(body: &[IfNode], initial_conditions: Predicate) -> Vec<Predicate> {
    let body_conditions = body.iter().flat_map(traverse_if_node);
    body_conditions
        .map(|body_condition| ir::Predicate::Group {
            left: Box::new(initial_conditions.clone()),
            right: Box::new(body_condition),
            operator: BoolOp::And,
        })
        .collect()
}

fn traverse_if_node(if_node: &ast::IfNode) -> Vec<ir::Predicate> {
    let initial_conditions = convert_condition_node(&if_node.conditions);

    let mut predicates_so_far = match &if_node.body {
        None => vec![initial_conditions],
        Some(body) if body.is_empty() => vec![initial_conditions],
        Some(body) => traverse_body(body, initial_conditions),
    };

    for else_if_node in if_node.else_if.iter() {
        let initial_conditions = convert_condition_node(&else_if_node.conditions);
        let previous_negated_plus_initial_conditions: Vec<Predicate> = predicates_so_far
            .iter()
            .map(|p| ir::Predicate::Group {
                left: Box::new(ir::Predicate::Negated(Box::new(p.clone()))),
                right: Box::new(initial_conditions.clone()),
                operator: BoolOp::And,
            })
            .collect();

        let mut else_if_predicates = match &else_if_node.body[..] {
            [] => previous_negated_plus_initial_conditions,
            body => previous_negated_plus_initial_conditions
                .iter()
                .flat_map(|p| traverse_body(body, p.clone()))
                .collect(),
        };

        predicates_so_far.append(&mut else_if_predicates);
    }

    let mut else_predicates = {
        let previous_negated: Vec<Predicate> = predicates_so_far
            .iter()
            .map(|p| ir::Predicate::Negated(Box::new(p.clone())))
            .collect();

        match &if_node.else_node {
            None => vec![],
            Some(ElseNode { body }) if body.is_empty() => previous_negated,
            Some(ElseNode { body }) => previous_negated
                .iter()
                .flat_map(|if_predicate| traverse_body(body, if_predicate.clone()))
                .collect(),
        }
    };

    predicates_so_far.append(&mut else_predicates);

    predicates_so_far
}

fn convert_variable<'a>(var_node: &'a ast::VarNode) -> ir::Variable {
    ir::Variable {
        var_name: var_node.var_name.to_owned(),
        var_type: var_node.var_type,
    }
}

fn traverse_feature_node(feature_node: &ast::FeatureNode) -> ir::Feature {
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

pub fn convert_ast_to_ir<'a>(root: &'a RootNode<'a>) -> Vec<ir::Feature> {
    root.features.iter().map(traverse_feature_node).collect()
}
