use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::char,
    combinator::{complete, cut, value},
    error::context,
    multi::many0,
    sequence::{delimited, terminated, tuple},
};

use super::IResult;

/// Parses a line cumment until the end of line, including \n.
/// Line comment starts with //
fn line_comment(input: &str) -> IResult<()> {
    context(
        "line comment",
        value(
            (), // Output is thrown away.
            tuple((tag("//"), cut(take_until("\n")), tag("\n"))),
        ),
    )(input)
}

fn multi_line_comment(input: &str) -> IResult<()> {
    context(
        "multi line comment",
        value(
            (), // Output is thrown away.
            tuple((tag("/*"), cut(take_until("*/")), tag("*/"))),
        ),
    )(input)
}

pub fn whitespace(input: &str) -> IResult<()> {
    let one_whitespace = complete(value(
        (),
        alt((char('\r'), char('\n'), char(' '), char('\t'))),
    ));
    value(
        (),
        many0(alt((one_whitespace, line_comment, multi_line_comment))),
    )(input)
}

pub fn token<'a, T>(
    mut parser: impl FnMut(&'a str) -> IResult<T>,
) -> impl FnMut(&'a str) -> IResult<T> {
    move |input| terminated(&mut parser, whitespace)(input)
}

pub fn token_lit(literal: &'static str) -> impl FnMut(&str) -> IResult<()> {
    move |input| value((), token(tag(literal)))(input)
}

pub fn parenthesized<'a, T>(
    mut parser: impl FnMut(&'a str) -> IResult<T>,
) -> impl FnMut(&'a str) -> IResult<T> {
    move |input| delimited(token(char('(')), &mut parser, token(char(')')))(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("//", None)]
    #[case("// asd", None)]
    #[case("// asd \nfoo", Some("foo"))]
    #[case("// asd \n foo", Some(" foo"))]
    #[case("// asd \r\nfoo", Some("foo"))]
    #[case("// asd \r\n foo", Some(" foo"))]
    #[case("//\n", Some(""))]
    #[case("//\nfoo", Some("foo"))]
    #[case("", None)]
    #[case("foo", None)]
    fn test_line_comment(#[case] input: &str, #[case] expected: Option<&str>) {
        match expected {
            Some(expected) => assert_eq!(line_comment(input), Ok((expected, ()))),
            None => assert!(line_comment(input).is_err()),
        }
    }

    #[rstest]
    #[case("/**/", Some(""))]
    #[case("/*    */", Some(""))]
    #[case("/*  \n  */", Some(""))]
    #[case("/*  \r\n  */", Some(""))]
    #[case("/*  \r\n \t \r\n \n \n */", Some(""))]
    #[case("/* asd qwe */ foo", Some(" foo"))]
    #[case("/* asd qwe */foo", Some("foo"))]
    #[case("/* asd qwe */\n foo", Some("\n foo"))]
    #[case("/* asd qwe */\n foo", Some("\n foo"))]
    #[case("/* not closed \n \r", None)]
    #[case("", None)]
    #[case("foo", None)]
    fn test_multi_line_comment(#[case] input: &str, #[case] expected: Option<&str>) {
        match expected {
            Some(expected) => assert_eq!(multi_line_comment(input), Ok((expected, ()))),
            None => assert!(multi_line_comment(input).is_err()),
        }
    }

    #[rstest]
    #[case("     ", Some(""))]
    #[case("     asd ", Some("asd "))]
    #[case("qwe", Some("qwe"))]
    #[case("", Some(""))]
    #[case("\n\t\n\t\t  \t  \n", Some(""))]
    #[case("\n\t\n\t\t // asd \n \t  \n", Some(""))]
    #[case("\n\t//\n\t\t // asd \n \t  \n", Some(""))]
    #[case("\n\t// foo bar baz\n//qwe\t\t // asd \n \t  \n", Some(""))]
    #[case("// foo bar baz\n  a // bar", Some("a // bar"))]
    #[case("/* asd qwe \n \t *///asd\n/*asd*/", Some(""))]
    #[case("/*asd*/ = 8", Some("= 8"))]
    #[case("// asd /* \n foo", Some("foo"))]
    fn test_whitespace(#[case] input: &str, #[case] expected: Option<&str>) {
        match expected {
            Some(expected) => assert_eq!(whitespace(input), Ok((expected, ()))),
            None => assert!(whitespace(input).is_err()),
        }
    }
}
