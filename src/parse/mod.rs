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
        println!("parse_program: {}", ptr);
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
pub fn parse_function<'a>(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<AFunction, ParseError> {
        println!("parse_function<'a>: {}", ptr);
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
pub fn parse_statement<'a>(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<AStatement, ParseError> {
        println!("parse_statement<'a>: {}", ptr);
        is_token(tokens, TokenType::Return, ptr)?;
        let expr = parse_expression(tokens, ptr)?;
        is_token(tokens, TokenType::SemiColon, ptr)?;

        Ok(AStatement { expr })
}

// <exp> ::= <int> | <unop> <exp> | "(" <exp> ")"
pub fn parse_expression<'a>(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<AExpression, ParseError> {
        println!("parse_expression<'a>: {}", ptr);
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
pub fn parse_unary_operator<'a>(tokens: &mut Vec<Token>, ptr: &mut usize) -> Option<Unop> {
        println!("parse_unary_operator<'a>: {}", ptr);
        if let Ok(_) = is_token(tokens, TokenType::Minus, ptr) {
                return Some(Unop::Negate);
        }
        if let Ok(_) = is_token(tokens, TokenType::Tilde, ptr) {
                return Some(Unop::Complement);
        }
        None
}

// <identifier> ::= ? An identifier token ?
pub fn parse_identifier<'a>(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<AIdentifier, ParseError> {
        println!("parse_identifier<'a>: {}", ptr);
        let (start, len) = is_token(tokens, TokenType::Identifier, ptr)?;

        return Ok(AIdentifier { start, len });
}

// <int> ::= ? A constant token ?
pub fn parse_constant<'a>(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<AConstant, ParseError> {
        println!("parse_constant<'a>: {}", ptr);
        let (start, len) = is_token(tokens, TokenType::Constant, ptr)?;

        return Ok(AConstant { start, len });
}

fn is_token<'a>(tokens: &mut Vec<Token>, wanted_token_type: TokenType, ptr: &mut usize) -> Result<(usize, usize), ParseError> {
        let Some(&Token { token_type, len, start }) = tokens.get(*ptr) else {
                return Err(ParseError::NotEnoughTokens);
        };
        println!("istoken: {:?}, ptr: {}", wanted_token_type, ptr);

        if token_type == wanted_token_type {
                *ptr += 1;
                return Ok((start, len));
        }
        Err(ParseError::InvalidTokenAt(start))
}
