pub mod nodes;

use crate::{
        lex::{tokentype::TokenType, Token},
        Lexed, Parsed, Program,
};
use nodes::{AConstant, AExpression, AFunction, AIdentifier, AProgram, AStatement, ReturnExpression};
use thiserror::Error;

impl Program<Lexed> {
        pub fn parse(self) -> Result<Program<Parsed>, ParseError> {
                Ok(Program {
                        state: Parsed {
                                pre_processor_output: self.state.pre_processor_output,
                                program: parse_program(&self.state.tokens)?,
                        },
                        ..self
                })
        }
}

#[derive(Debug, Error)]
pub enum ParseError {
        #[error("Error is at {0}")]
        At(usize),
        #[error("Your Program just ends abruptly.")]
        OutOfTokens,
        #[error("Your Program has extra junk in it.")]
        TooManyTokens,
}

fn check_token_type<'a>(
        tokens: &mut impl Iterator<Item = &'a Token>,
        token_type: &TokenType,
) -> Result<(), ParseError> {
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        if !(&pot.token_type == token_type) {
                return Err(ParseError::At(pot.start));
        }
        Ok(())
}

fn check_token_types<'a>(
        tokens: &mut impl Iterator<Item = &'a Token>,
        token_types: &[TokenType],
) -> Result<(), ParseError> {
        for i in token_types {
                check_token_type(&mut *tokens, i)?;
        }
        Ok(())
}

// <program> ::= <function>
pub fn parse_program(tokens: &Vec<Token>) -> Result<AProgram, ParseError> {
        let mut tokensiter = tokens.iter();
        let function = parse_function(&mut tokensiter)?;
        if let Some(_) = tokensiter.next() {
                return Err(ParseError::TooManyTokens);
        }

        Ok(AProgram { function })
}

// <function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
fn parse_function<'a>(tokens: &mut impl Iterator<Item = &'a Token>) -> Result<AFunction, ParseError> {
        check_token_type(&mut *tokens, &TokenType::KeywordInt)?;

        let identifier = parse_identifier(&mut *tokens)?;

        check_token_types(
                &mut *tokens,
                &[
                        TokenType::OpenParanthesis,
                        TokenType::KeywordVoid,
                        TokenType::CloseParanthesis,
                        TokenType::OpenBrace,
                ],
        )?;

        let statement_body = parse_statement(&mut *tokens)?;

        check_token_type(&mut *tokens, &TokenType::CloseBrace)?;

        Ok(AFunction {
                identifier,
                statement_body,
        })
}

// <identifier> ::= ? A identifier token ?
fn parse_identifier<'a>(mut tokens: impl Iterator<Item = &'a Token>) -> Result<AIdentifier, ParseError> {
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        let Token {
                token_type: TokenType::Identifier(len),
                start,
        } = *pot
        else {
                return Err(ParseError::At(pot.start));
        };

        Ok(AIdentifier { len, start })
}

// <statement> ::= "return" <exp> ";"
fn parse_statement<'a>(mut tokens: impl Iterator<Item = &'a Token>) -> Result<AStatement, ParseError> {
        check_token_type(&mut tokens, &TokenType::KeywordReturn)?;

        let exp = parse_expression(&mut tokens)?;

        check_token_type(&mut tokens, &TokenType::SemiColon)?;

        Ok(AStatement::ReturnStatement(exp))
}

// <exp> ::= <int>
// <int> ::= ? A constant token ?
fn parse_expression<'a>(
        mut tokens: impl Iterator<Item = &'a Token>,
) -> Result<AExpression<ReturnExpression>, ParseError> {
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;

        let (TokenType::Constant(len), start) = (pot.token_type, pot.start) else {
                return Err(ParseError::At(pot.start));
        };

        Ok(AExpression {
                state: ReturnExpression {
                        constant: AConstant { len, start },
                },
        })
}
