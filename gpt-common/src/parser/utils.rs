use nom::{
    branch::alt,
    character::complete::char,
    combinator::{complete, value},
    multi::many0,
    sequence::{delimited, terminated},
};

use super::IResult;

pub fn comment(input: &str) -> IResult<()> {
    todo!("Implement comment parsing")
}

pub fn whitespace(input: &str) -> IResult<()> {
    let one_whitespace = complete(alt((char('\n'), char(' '), char('\t'))));
    value((), many0(one_whitespace))(input)
}

pub fn parenthesized<'a, T>(
    mut parser: impl FnMut(&'a str) -> IResult<T>,
) -> impl FnMut(&'a str) -> IResult<T> {
    move |input| {
        delimited(
            terminated(char('('), whitespace),
            |i| parser(i),
            terminated(char(')'), whitespace),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace("     "), Ok(("", ())));
        assert_eq!(whitespace("     asd "), Ok(("asd ", ())));
        assert_eq!(whitespace("qwe"), Ok(("qwe", ())));
        assert_eq!(whitespace(""), Ok(("", ())));
        assert_eq!(whitespace("\n\t\n\t\t  \t  \n"), Ok(("", ())));
    }
}
