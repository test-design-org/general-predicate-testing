use nom::{
    branch::alt,
    combinator::{complete, cut},
    error::context,
};

use super::{
    ast::{Type, VarNode},
    primitives::{float, var_name},
    utils::{token, token_lit},
    IResult,
};

fn parse_float_type(input: &str) -> IResult<Type> {
    let (input, _) = token_lit("num")(input)?;
    let (input, _) = token_lit("(")(input)?;
    cut(|input| {
        let (input, precision) = token(float)(input)?;
        let (input, _) = token_lit(")")(input)?;

        Ok((input, Type::Float { precision }))
    })(input)
}

fn parse_bool_type(input: &str) -> IResult<Type> {
    let (input, _) = token_lit("bool")(input)?;

    Ok((input, Type::Bool))
}
fn parse_int_type(input: &str) -> IResult<Type> {
    let (input, _) = token_lit("int")(input)?;

    Ok((input, Type::Integer))
}

fn parse_simple_num_type(input: &str) -> IResult<Type> {
    let (input, _) = token_lit("num")(input)?;

    // TODO: This should be a default num precision somewhere
    Ok((input, Type::Float { precision: 0.01 }))
}

fn parse_type(input: &str) -> IResult<Type> {
    context(
        "type",
        alt((
            parse_bool_type,
            parse_int_type,
            complete(parse_float_type),
            parse_simple_num_type,
        )),
    )(input)
}

pub fn var_declaration(input: &str) -> IResult<VarNode> {
    context("var declaration", |input| {
        let (input, _) = token_lit("var")(input)?;
        cut(|input| {
            let (input, var_name) = token(var_name)(input)?;
            let (input, _) = token_lit(":")(input)?;
            let (input, var_type) = token(parse_type)(input)?;

            Ok((input, VarNode { var_name, var_type }))
        })(input)
    })(input)
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "todo"]
    fn test_parse_float_type() {
        todo!("Write tests for parse_float_type");
    }

    #[test]
    #[ignore = "todo"]
    fn test_parse_bool_type() {
        todo!("Write tests for parse_bool_type");
    }

    #[test]
    #[ignore = "todo"]
    fn test_parse_int_type() {
        todo!("Write tests for parse_int_type");
    }

    #[test]
    #[ignore = "todo"]
    fn test_parse_simple_num_type() {
        todo!("Write tests for parse_simple_num_type");
    }

    #[test]
    #[ignore = "todo"]
    fn test_parse_type() {
        todo!("Write tests for parse_type");
    }

    #[test]
    #[ignore = "todo"]
    fn test_var_declaration() {
        todo!("Write tests for var_declaration");
    }
}
