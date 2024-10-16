use nodes::{AConstant, AExpression, AFunction, AIdentifier, AProgram, AStatement};
use thiserror::Error;

use crate::{
        lex::{
                tokentype::{Token, TokenType},
                Lexed,
        },
        Program, State,
};

pub mod nodes;

#[derive(Debug)]
pub struct Parsed {
        pub code: Vec<u8>,
        pub program: AProgram,
}
impl State for Parsed {}

#[derive(Debug, Error)]
pub enum ParseError {
        #[error("Not enough tokens for a full program")]
        NotEnoughTokens,
        #[error("Invalid token at {0}")]
        InvalidTokenAt(usize),
        #[error("Too many tokens: You have some junk after the program")]
        TooManyTokens,
}

// <program> ::= <function>
pub fn parse_program(program: Program<Lexed>) -> Result<Program<Parsed>, ParseError> {
        let mut tokens_iter = program.state.tokens.iter();
        let afunction = parse_function(&mut tokens_iter)?;

        if tokens_iter.next().is_some() {
                return Err(ParseError::TooManyTokens);
        }

        Ok(Program {
                state: Parsed {
                        code: program.state.code,
                        program: AProgram { functions: vec![afunction] },
                },
                operation: program.operation,
        })
}

// <function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
pub fn parse_function<'a>(tokens_iter: &mut impl Iterator<Item = &'a Token>) -> Result<AFunction, ParseError> {
        is_token(tokens_iter, TokenType::Int)?;
        let identifier = parse_identifier(tokens_iter)?;
        is_token(tokens_iter, TokenType::OpenParen)?;
        is_token(tokens_iter, TokenType::Void)?;
        is_token(tokens_iter, TokenType::CloseParen)?;
        is_token(tokens_iter, TokenType::OpenBrace)?;
        let statement_body = parse_statement(tokens_iter)?;
        is_token(tokens_iter, TokenType::CloseBrace)?;

        Ok(AFunction { identifier, statement_body })
}

// <statement> ::= "return" <exp> ";"
pub fn parse_statement<'a>(tokens_iter: &mut impl Iterator<Item = &'a Token>) -> Result<AStatement, ParseError> {
        is_token(tokens_iter, TokenType::Return)?;
        let expr = parse_expression(tokens_iter)?;
        is_token(tokens_iter, TokenType::SemiColon)?;

        Ok(AStatement { expr })
}

// <exp> ::= <int>
pub fn parse_expression<'a>(tokens_iter: &mut impl Iterator<Item = &'a Token>) -> Result<AExpression, ParseError> { Ok(AExpression(parse_constant(tokens_iter)?)) }

// <identifier> ::= ? An identifier token ?
pub fn parse_identifier<'a>(tokens_iter: &mut impl Iterator<Item = &'a Token>) -> Result<AIdentifier, ParseError> {
        let (start, len) = is_token(tokens_iter, TokenType::Identifier)?;

        return Ok(AIdentifier { start, len });
}

// <int> ::= ? A constant token ?
pub fn parse_constant<'a>(tokens_iter: &mut impl Iterator<Item = &'a Token>) -> Result<AConstant, ParseError> {
        let (start, len) = is_token(tokens_iter, TokenType::Constant)?;

        return Ok(AConstant { start, len });
}

fn is_token<'a>(tokens_iter: &mut impl Iterator<Item = &'a Token>, wanted_token_type: TokenType) -> Result<(usize, usize), ParseError> {
        let Some(&Token { token_type, len, start }) = tokens_iter.next() else {
                return Err(ParseError::NotEnoughTokens);
        };

        if token_type == wanted_token_type {
                return Ok((start, len));
        }
        Err(ParseError::InvalidTokenAt(start))
}
