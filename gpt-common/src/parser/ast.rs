use crate::interval::MultiInterval;

pub enum Type {
    Bool,
    Integer,
    Float { precision: f32 },
}

#[derive(Debug, PartialEq, Clone)]
pub enum EqOp {
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    LessThan,
    GreaterThan,
    LessThanEqualTo,
    GreaterThanEqualTo,
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq)]
pub enum IntervalOp {
    In,
    NotIn,
}

#[derive(Debug, PartialEq)]
pub enum BoolOp {
    And,
    // Or,
}

pub enum ConstantPosition {
    LeftHandSide,
    RightHandSide,
}

pub struct BoolCondition<'a> {
    pub var_name: &'a str,
    pub constant: bool,
    pub eq_op: EqOp,
}

pub struct BinaryCondition<'a> {
    var_name: &'a str,
    constant_position: ConstantPosition,
    constant: f32,
    binary_op: BinaryOp,
}
pub struct IntervalCondition<'a> {
    var_name: &'a str,
    interval_op: IntervalOp,
    interval: MultiInterval,
}

pub enum Condition<'a> {
    Bool(BoolCondition<'a>),
    Binary(BinaryCondition<'a>),
    Interval(IntervalCondition<'a>),
}

pub struct FeatureNode<'a> {
    variables: Vec<VarNode<'a>>,
    if_statements: Vec<IfNode<'a>>,
    features: Vec<FeatureNode<'a>>,
}

pub struct VarNode<'a> {
    var_name: &'a str,
    var_type: Type,
}

pub struct IfNode<'a> {
    conditions: ConditionsNode<'a>,
    body: Option<Vec<IfNode<'a>>>,
    else_if: Option<Vec<ElseIfNode<'a>>>,
    else_node: Option<ElseNode<'a>>,
}

pub struct ElseIfNode<'a> {
    conditions: ConditionsNode<'a>,
}

pub struct ElseNode<'a> {
    body: Vec<IfNode<'a>>,
}

pub struct ConditionsNode<'a> {
    conditions: Vec<Condition<'a>>,
}
