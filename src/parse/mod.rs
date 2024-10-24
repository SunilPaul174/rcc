use nodes::{AConstant, AExpression, AFactor, AFunction, AIdentifier, AProgram, AStatement, BinOp, Unop};
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
pub enum Error {
        #[error("Not enough tokens for a full program")]
        NotEnoughTokens,
        #[error("Invalid token {0}. Expected {1}")]
        InvalidTokenAt(Token, TokenType),
        #[error("Too many tokens: You have some junk after the program")]
        TooManyTokens,
}

// <program> ::= <function>
pub fn parse_program(mut program: Program<Lexed>) -> Result<Program<Parsed>, Error> {
        let mut ptr = 0;
        let functions = parse_function(&mut program.state.tokens, &mut ptr)?;

        if ptr < program.state.tokens.len() {
                return Err(Error::TooManyTokens);
        }

        Ok(Program {
                state: Parsed {
                        code: program.state.code,
                        program: AProgram { functions },
                },
                operation: program.operation,
        })
}

// <function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
fn parse_function(tokens: &mut [Token], ptr: &mut usize) -> Result<AFunction, Error> {
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
fn parse_statement(tokens: &mut [Token], ptr: &mut usize) -> Result<AStatement, Error> {
        is_token(tokens, TokenType::Return, ptr)?;
        let expr = parse_expression(tokens, ptr, 0)?;
        is_token(tokens, TokenType::SemiColon, ptr)?;

        Ok(AStatement { expr })
}

// <exp> ::= <factor> | <exp> <binop> <exp>
fn parse_expression(tokens: &mut [Token], ptr: &mut usize, min_prec: usize) -> Result<AExpression, Error> {
        let mut left = AExpression::Factor(parse_factor(tokens, ptr)?);

        while let Some(operator) = parse_binary_operator(tokens, ptr) {
                if precedence(operator) < min_prec {
                        *ptr -= 1;
                        return Ok(left);
                }

                let right = parse_expression(tokens, ptr, precedence(operator) + 1)?;
                left = AExpression::BinOp(operator, Box::new(left), Box::new(right));
        }

        Ok(left)
}

// <factor> ::= <int> | <unop> <factor> | "(" <exp> ")"
fn parse_factor(tokens: &mut [Token], ptr: &mut usize) -> Result<AFactor, Error> {
        if let Ok(constant) = parse_constant(tokens, ptr) {
                return Ok(AFactor::Constant(constant));
        }

        if let Ok(_) = is_token(tokens, TokenType::OpenParen, ptr) {
                if let Ok(expr) = parse_expression(tokens, ptr, 0) {
                        if let Ok(_) = is_token(tokens, TokenType::CloseParen, ptr) {
                                return Ok(AFactor::Expr(Box::new(expr)));
                        }
                        *ptr -= 1;
                }
                *ptr -= 1;
        }

        if let Some(unop) = parse_unary_operator(tokens, ptr) {
                if let Ok(factor) = parse_factor(tokens, ptr) {
                        return Ok(AFactor::Unop(unop, Box::new(factor)));
                }
                *ptr -= 1;
        }

        Err(Error::InvalidTokenAt(tokens[*ptr], TokenType::Int))
}

// <unop> ::= "-" | "~" | "!"
fn parse_unary_operator(tokens: &mut [Token], ptr: &mut usize) -> Option<Unop> {
        if let Some(unop) = match tokens[*ptr].token_type {
                TokenType::Minus => Some(Unop::Negate),
                TokenType::Tilde => Some(Unop::Complement),
                TokenType::Not => Some(Unop::Not),
                _ => None,
        } {
                *ptr += 1;
                Some(unop)
        } else {
                None
        }
}

// <binop> ::= "-" | "+" | "*" | "/" | "%" | "<<" | ">>" | "&" | "|" | | "^"
// | "&&" | "||" | "==" | "!=" | "<" | "<=" | ">" | ">="
fn parse_binary_operator(tokens: &mut [Token], ptr: &mut usize) -> Option<BinOp> {
        if let Some(binop) = match tokens[*ptr].token_type {
                TokenType::Minus => Some(BinOp::Subtract),
                TokenType::Plus => Some(BinOp::Add),
                TokenType::Asterisk => Some(BinOp::Multiply),
                TokenType::ForwardSlash => Some(BinOp::Divide),
                TokenType::Percent => Some(BinOp::Remainder),
                TokenType::BitwiseAnd => Some(BinOp::BitwiseAnd),
                TokenType::LogicalAnd => Some(BinOp::LogicalAnd),
                TokenType::BitwiseOr => Some(BinOp::BitwiseOr),
                TokenType::LogicalOr => Some(BinOp::LogicalOr),
                TokenType::BitwiseXOr => Some(BinOp::BitwiseXOr),
                TokenType::LeftShift => Some(BinOp::LeftShift),
                TokenType::RightShift => Some(BinOp::RightShift),
                TokenType::LessThan => Some(BinOp::LessThan),
                TokenType::MoreThan => Some(BinOp::MoreThan),
                TokenType::LessThanOrEqual => Some(BinOp::LessThanOrEqual),
                TokenType::MoreThanOrEqual => Some(BinOp::MoreThanOrEqual),
                TokenType::EqualTo => Some(BinOp::EqualTo),
                TokenType::NotEqualTo => Some(BinOp::NotEqualTo),
                _ => None,
        } {
                *ptr += 1;
                Some(binop)
        } else {
                None
        }
}

// <identifier> ::= ? An identifier token ?
fn parse_identifier(tokens: &mut [Token], ptr: &mut usize) -> Result<AIdentifier, Error> {
        let (start, len) = is_token(tokens, TokenType::Identifier, ptr)?;

        Ok(AIdentifier { start, len })
}

// <int> ::= ? A constant token ?
fn parse_constant(tokens: &mut [Token], ptr: &mut usize) -> Result<AConstant, Error> {
        let (start, len) = is_token(tokens, TokenType::Constant, ptr)?;

        Ok(AConstant { start, len })
}

fn is_token(tokens: &mut [Token], wanted_token_type: TokenType, ptr: &mut usize) -> Result<(usize, usize), Error> {
        let Some(&Token { token_type, len, start }) = tokens.get(*ptr) else {
                return Err(Error::NotEnoughTokens);
        };

        if token_type == wanted_token_type {
                *ptr += 1;
                return Ok((start, len));
        }

        Err(Error::InvalidTokenAt(tokens[*ptr], wanted_token_type))
}

fn precedence(operator: BinOp) -> usize {
        match operator {
                BinOp::Multiply => 50,
                BinOp::Divide => 50,
                BinOp::Remainder => 50,
                BinOp::Add => 45,
                BinOp::Subtract => 45,
                BinOp::LeftShift => 37,
                BinOp::RightShift => 37,
                BinOp::LessThan => 35,
                BinOp::LessThanOrEqual => 35,
                BinOp::MoreThan => 35,
                BinOp::MoreThanOrEqual => 35,
                BinOp::EqualTo => 30,
                BinOp::NotEqualTo => 30,
                BinOp::BitwiseAnd => 20,
                BinOp::BitwiseXOr => 17,
                BinOp::BitwiseOr => 15,
                BinOp::LogicalAnd => 10,
                BinOp::LogicalOr => 5,
        }
}
