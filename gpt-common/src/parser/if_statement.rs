use nom::{
    combinator::{cut, map, opt},
    error::context,
    multi::{many0, many1},
    sequence::tuple,
};

use super::{
    ast::{ElseIfNode, ElseNode, IfNode},
    condition::conditions,
    utils::token_lit,
    IResult,
};

fn else_if_statement(input: &str) -> IResult<ElseIfNode> {
    context("else if statement", |input| {
        let (input, _) = token_lit("else")(input)?;
        let (input, _) = token_lit("if")(input)?;
        let (input, _) = token_lit("(")(input)?;
        let (input, conditions) = conditions(input)?;
        let (input, _) = token_lit(")")(input)?;
        let (input, body) = opt(map(
            tuple((token_lit("{"), many0(if_statement), token_lit("}"))),
            |(_, body, _)| body,
        ))(input)?;

        let else_if_node = ElseIfNode {
            conditions,
            body: body.unwrap_or_default(),
        };

        Ok((input, else_if_node))
    })(input)
}

fn else_statement(input: &str) -> IResult<ElseNode> {
    context("else statement", |input| {
        let (input, _) = token_lit("else")(input)?;
        let (input, if_statements) = opt(|input| {
            let (input, _) = token_lit("{")(input)?;
            let (input, if_statements) = many0(if_statement)(input)?;
            let (input, _) = token_lit("}")(input)?;

            Ok((input, if_statements))
        })(input)?;

        let else_node = ElseNode {
            body: if_statements.unwrap_or_default(),
        };

        Ok((input, else_node))
    })(input)
}

pub fn if_statement(input: &str) -> IResult<IfNode> {
    context("if statement", |input| {
        let (input, _) = token_lit("if")(input)?;
        cut(|input| {
            let (input, _) = token_lit("(")(input)?;
            let (input, conditions) = conditions(input)?;
            let (input, _) = token_lit(")")(input)?;
            let (input, body) = opt(map(
                tuple((token_lit("{"), many0(if_statement), token_lit("}"))),
                |(_, body, _)| body,
            ))(input)?;
            // TODO: would a many0 work here? It'd be better
            let (input, else_if_statements) = opt(many1(else_if_statement))(input)?;
            let (input, else_statement) = opt(else_statement)(input)?;

            let if_node = IfNode {
                body,
                conditions,
                else_if: else_if_statements.unwrap_or_default(),
                else_node: else_statement,
            };

            Ok((input, if_node))
        })(input)
    })(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_if_statement() {
        assert_eq!(
            if_statement(
                "
            if (x >= 5 && y in (0, 10)) {
                if (x == true)
            } else if (x < 4 && y > 6)
            else {
                if (x != false)
            }   qwe
        "
                .trim()
            ),
            Ok((
                "qwe",
                IfNode {
                    conditions: conditions("x >= 5 && y in (0, 10)").unwrap().1,
                    body: Some(vec![IfNode {
                        conditions: conditions("x == true").unwrap().1,
                        body: None,
                        else_if: vec![],
                        else_node: None
                    }]),
                    else_if: vec![ElseIfNode {
                        conditions: conditions("x < 4 && y > 6").unwrap().1,
                        body: vec![]
                    }],
                    else_node: Some(ElseNode {
                        body: vec![IfNode {
                            conditions: conditions("x != false").unwrap().1,
                            body: None,
                            else_if: vec![],
                            else_node: None
                        }]
                    })
                }
            ))
        );
        // TODO
    }

    #[test]
    #[ignore = "todo"]
    fn test_else_if_statement() {
        // TODO
        todo!()
    }

    #[test]
    #[ignore = "todo"]
    fn test_else_statement() {
        // TODO
        todo!()
    }
}
