pub mod ast;
mod condition;
mod feature;
mod if_statement;
mod primitives;
mod utils;
mod var_declaration;

use ast::RootNode;
use nom::{
    branch::alt,
    combinator::{eof, map},
    error::VerboseError,
    multi::many1,
};

pub use self::primitives::interval;
use self::{
    feature::{feature, feature_body},
    utils::{token, whitespace},
};
use super::dto::NTupleInput;
use crate::ir;

type IResult<'a, O> = nom::IResult<&'a str, O, VerboseError<&'a str>>;

fn root(input: &str) -> IResult<RootNode> {
    let (input, _) = whitespace(input)?;

    // Accept empty files as input
    if eof::<&str, VerboseError<&str>>(input).is_ok() {
        return Ok((input, RootNode { features: vec![] }));
    }

    let (input, features) = alt((
        many1(token(feature)),                 // Either a list of HGPT features
        map(token(feature_body), |x| vec![x]), // Or a sungle feature without the brackets
    ))(input)?;
    let (input, _) = eof(input)?;

    Ok((input, RootNode { features }))
}

pub fn parse_gpt_to_features(input: &str) -> IResult<Vec<Vec<NTupleInput>>> {
    let (input, ast) = root(input)?;
    let ir_features = ir::ast_to_ir::convert_ast_to_ir(&ast);
    let ntuples_for_features = ir_features
        .iter()
        .map(ir::ir_to_ntuple::ir_to_ntuple)
        .collect();

    Ok((input, ntuples_for_features))
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "todo"]
    fn test_root() {
        todo!("Write tests for root parser")
    }

    #[test]
    #[ignore = "todo"]
    fn test_feature() {
        todo!("Write tests for feature parser")
    }
}
