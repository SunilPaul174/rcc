use nodes::{AConstant, AExpression, AFunction, AIdentifier, AProgram, AStatement, Unop};
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
pub fn parse_program(mut program: Program<Lexed>) -> Result<Program<Parsed>, ParseError> {
        // let mut tokens = program.state.tokens.iter();
        let mut ptr = 0;
        let afunction = parse_function(&mut program.state.tokens, &mut ptr)?;

        if ptr < (program.state.tokens.len() - 1) {
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
pub fn parse_function(tokens: &mut [Token], ptr: &mut usize) -> Result<AFunction, ParseError> {
        is_token(tokens, TokenType::Int, ptr)?;
        let identifier = parse_identifier(tokens, ptr)?;
        is_token(tokens, TokenType::OpenParen, ptr)?;
        is_token(tokens, TokenType::Void, ptr)?;
        is_token(tokens, TokenType::CloseParen, ptr)?;
        is_token(tokens, TokenType::OpenBrace, ptr)?;
        let statement_body = parse_statement(tokens, ptr)?;
        is_token(tokens, TokenType::CloseBrace, ptr)?;

        Ok(AFunction { identifier, statement_body })
}

// <statement> ::= "return" <exp> ";"
pub fn parse_statement(tokens: &mut [Token], ptr: &mut usize) -> Result<AStatement, ParseError> {
        is_token(tokens, TokenType::Return, ptr)?;
        let expr = parse_expression(tokens, ptr)?;
        is_token(tokens, TokenType::SemiColon, ptr)?;

        Ok(AStatement { expr })
}

// <exp> ::= <int> | <unop> <exp> | "(" <exp> ")"
pub fn parse_expression(tokens: &mut [Token], ptr: &mut usize) -> Result<AExpression, ParseError> {
        if let Ok(constant) = parse_constant(tokens, ptr) {
                return Ok(AExpression::Constant(constant));
        }

        if let (Some(unop), Ok(expr)) = (parse_unary_operator(tokens, ptr), parse_expression(tokens, ptr)) {
                return Ok(AExpression::Unop(unop, Box::new(expr)));
        }

        if let (Ok(_), Ok(expr), Ok(_)) = (
                is_token(tokens, TokenType::OpenParen, ptr),
                parse_expression(tokens, ptr),
                is_token(tokens, TokenType::CloseParen, ptr),
        ) {
                return Ok(expr);
        }

        Err(ParseError::InvalidTokenAt(*ptr))
}

// <unop> ::= "-" | "~"
pub fn parse_unary_operator(tokens: &mut [Token], ptr: &mut usize) -> Option<Unop> {
        if is_token(tokens, TokenType::Minus, ptr).is_ok() {
                Some(Unop::Negate)
        } else if is_token(tokens, TokenType::Tilde, ptr).is_ok() {
                Some(Unop::Complement)
        } else {
                None
        }
}

// <identifier> ::= ? An identifier token ?
pub fn parse_identifier(tokens: &mut [Token], ptr: &mut usize) -> Result<AIdentifier, ParseError> {
        let (start, len) = is_token(tokens, TokenType::Identifier, ptr)?;

        Ok(AIdentifier { start, len })
}

// <int> ::= ? A constant token ?
pub fn parse_constant(tokens: &mut [Token], ptr: &mut usize) -> Result<AConstant, ParseError> {
        let (start, len) = is_token(tokens, TokenType::Constant, ptr)?;

        Ok(AConstant { start, len })
}

fn is_token(tokens: &mut [Token], wanted_token_type: TokenType, ptr: &mut usize) -> Result<(usize, usize), ParseError> {
        let Some(&Token { token_type, len, start }) = tokens.get(*ptr) else {
                return Err(ParseError::NotEnoughTokens);
        };

        if token_type == wanted_token_type {
                *ptr += 1;
                return Ok((start, len));
        }
        Err(ParseError::InvalidTokenAt(start))
}
