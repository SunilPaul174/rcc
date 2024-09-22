pub mod nodes;

use crate::{
        lex::{tokentype::TokenType, Token},
        Lexed, Parsed, Program,
};
use nodes::{AExpression, AFunction, AIdentifier, AProgram, AStatement};
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
}

fn check_token_type(token_type: TokenType, pot: &Token) -> Result<(), ParseError> {
        if !(pot.token_type == token_type) {
                return Err(ParseError::At(pot.start));
        }
        Ok(())
}

// <program> ::= <function>
pub fn parse_program(tokens: &Vec<Token>) -> Result<AProgram, ParseError> {
        let function = parse_function(tokens.iter())?;

        Ok(AProgram { function })
}

// <function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
fn parse_function<'a>(mut tokens: impl Iterator<Item = &'a Token>) -> Result<AFunction, ParseError> {
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        check_token_type(TokenType::KeywordInt, &pot)?;

        let identifier = parse_identifier(&mut tokens)?;

        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        check_token_type(TokenType::OpenParanthesis, &pot)?;
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        check_token_type(TokenType::KeywordVoid, &pot)?;
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        check_token_type(TokenType::CloseParanthesis, &pot)?;
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        check_token_type(TokenType::OpenBrace, &pot)?;

        let statement_body = parse_statement(&mut tokens)?;

        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        check_token_type(TokenType::CloseBrace, &pot)?;

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

        Ok(AIdentifier((len, start)))
}

// <statement> ::= "return" <exp> ";"
fn parse_statement<'a>(mut tokens: impl Iterator<Item = &'a Token>) -> Result<AStatement, ParseError> {
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        check_token_type(TokenType::KeywordReturn, &pot)?;

        let exp = parse_expression(&mut tokens)?;

        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;
        check_token_type(TokenType::SemiColon, &pot)?;

        Ok(AStatement::ReturnStatement(exp))
}

// <exp> ::= <int>
// <int> ::= ? A constant token ?
fn parse_expression<'a>(mut tokens: impl Iterator<Item = &'a Token>) -> Result<AExpression, ParseError> {
        let pot = tokens.next().ok_or(ParseError::OutOfTokens)?;

        let TokenType::Constant(constant) = pot.token_type else {
                return Err(ParseError::At(pot.start));
        };

        Ok(AExpression::Constant(constant))
}
