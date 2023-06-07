use nom::{
    branch::alt,
    combinator::{cut, map},
    multi::many1,
};

use super::{
    ast::{FeatureNode, IfNode, VarNode},
    if_statement::if_statement,
    utils::token_lit,
    var_declaration::var_declaration,
    IResult,
};

pub fn feature_body(input: &str) -> IResult<FeatureNode> {
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

    let (input, nodes) = many1(var_or_if)(input)?;

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

pub fn feature(input: &str) -> IResult<FeatureNode> {
    let (input, _) = token_lit("[")(input)?;
    let (input, feature_node) = cut(feature_body)(input)?;
    let (input, _) = cut(token_lit("]"))(input)?;

    Ok((input, feature_node))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    // TODO: Make this test pass
    #[test]
    #[ignore = "todo"]
    fn test_feature() {
        let input = r#"
        [
            var VIP: bool 
            var price: num
            var second_hand_price: num
          
            if(VIP == true && price < 50 && price != 0) {
              if(price < 20 && second_hand_price > 60)
              if(price != 50)
            }
            else if(second_hand_price > 60)
          
            if(price > 30 && second_hand_price > 60)
          
            if(VIP == true) {
              if(second_hand_price == 2)
              if(second_hand_price == 3)
            }
          
            if(second_hand_price >= 50) {
              if(price < 5)
            }
            else if(10 < second_hand_price)
          
            if(price in [0,10] && price not in (9,100])
          
            if(VIP == true && price < 10) {
              if(second_hand_price == 2)
              if(second_hand_price == 3)
            }
            if(VIP == true) {
              if(second_hand_price == 2)
              if(second_hand_price == 3)
            }
            if(price < 10) {
              if(second_hand_price == 2)
              if(second_hand_price == 3)
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
