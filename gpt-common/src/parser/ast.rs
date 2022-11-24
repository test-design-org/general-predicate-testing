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

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    LessThan,
    GreaterThan,
    LessThanEqualTo,
    GreaterThanEqualTo,
    Equal,
    NotEqual,
}

impl BinaryOp {
    /// Swaps the `BinaryOp` as if the left and right hand side were swapped.
    ///
    /// Example: x > 10 == 10 < x, y = 20 == 20 = y
    pub fn flip(&self) -> BinaryOp {
        match self {
            BinaryOp::LessThan => BinaryOp::GreaterThan,
            BinaryOp::GreaterThan => BinaryOp::LessThan,
            BinaryOp::LessThanEqualTo => BinaryOp::GreaterThanEqualTo,
            BinaryOp::GreaterThanEqualTo => BinaryOp::LessThanEqualTo,
            BinaryOp::Equal => BinaryOp::Equal,
            BinaryOp::NotEqual => BinaryOp::NotEqual,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum IntervalOp {
    In,
    NotIn,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoolOp {
    And,
    // Or,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConstantPosition {
    LeftHandSide,
    RightHandSide,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BoolCondition<'a> {
    pub var_name: &'a str,
    pub constant: bool,
    pub eq_op: EqOp,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryCondition<'a> {
    pub var_name: &'a str,
    pub constant_position: ConstantPosition,
    pub constant: f32,
    pub binary_op: BinaryOp,
}
#[derive(Debug, PartialEq, Clone)]
pub struct IntervalCondition<'a> {
    pub var_name: &'a str,
    pub interval_op: IntervalOp,
    pub interval: MultiInterval,
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(PartialEq, Debug)]
pub struct IfNode<'a> {
    pub conditions: ConditionsNode<'a>,
    pub body: Option<Vec<IfNode<'a>>>,
    pub else_if: Option<Vec<ElseIfNode<'a>>>,
    pub else_node: Option<ElseNode<'a>>,
}

#[derive(PartialEq, Debug)]
pub struct ElseIfNode<'a> {
    pub conditions: ConditionsNode<'a>,
    pub body: Option<Vec<IfNode<'a>>>,
}

#[derive(PartialEq, Debug)]
pub struct ElseNode<'a> {
    pub body: Vec<IfNode<'a>>,
}

#[derive(PartialEq, Debug)]
pub struct ConditionsNode<'a> {
    pub conditions: Vec<Condition<'a>>,
}
