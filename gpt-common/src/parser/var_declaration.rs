use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{complete, cut},
    error::context,
    sequence::terminated,
};

use super::{
    ast::{Type, VarNode},
    primitives::{float, var_name},
    utils::whitespace,
    IResult,
};

fn parse_float_type(input: &str) -> IResult<Type> {
    let (input, _) = terminated(tag("num"), whitespace)(input)?;
    let (input, _) = terminated(tag("("), whitespace)(input)?;
    cut(|input| {
        let (input, precision) = terminated(float, whitespace)(input)?;
        let (input, _) = terminated(tag(")"), whitespace)(input)?;

        Ok((input, Type::Float { precision }))
    })(input)
}

fn parse_bool_type(input: &str) -> IResult<Type> {
    let (input, _) = terminated(tag("bool"), whitespace)(input)?;

    Ok((input, Type::Bool))
}
fn parse_int_type(input: &str) -> IResult<Type> {
    let (input, _) = terminated(tag("int"), whitespace)(input)?;

    Ok((input, Type::Integer))
}

fn parse_simple_num_type(input: &str) -> IResult<Type> {
    let (input, _) = terminated(tag("num"), whitespace)(input)?;

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
        let (input, _) = terminated(tag("var"), whitespace)(input)?;
        cut(|input| {
            let (input, var_name) = terminated(var_name, whitespace)(input)?;
            let (input, _) = terminated(tag(":"), whitespace)(input)?;
            let (input, var_type) = terminated(parse_type, whitespace)(input)?;

            Ok((input, VarNode { var_name, var_type }))
        })(input)
    })(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

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
