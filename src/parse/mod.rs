use nodes::{AConstant, AExpression, AFactor, AFunction, AIdentifier, AProgram, AStatement, Binop, BlockItem, Conditional, Declaration, IfStatement, Unop};
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
        #[error("Invalid token {0} to be a factor.")]
        InvalidFactorAt(Token),
}

// <program> ::= <function>
pub fn parse_program(mut program: Program<Lexed>) -> Result<Program<Parsed>, Error> {
        let mut ptr = 0;
        let function = parse_function(&mut program.state.tokens, &mut ptr)?;

        if ptr < program.state.tokens.len() {
                return Err(Error::TooManyTokens);
        }

        Ok(Program {
                state: Parsed {
                        code: program.state.code,
                        program: AProgram { function },
                },
                operation: program.operation,
        })
}

// <function> ::= "int" <identifier> "(" "void" ")" "{" { <block_item> } "}"
fn parse_function(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<AFunction, Error> {
        is_token(tokens, TokenType::Int, ptr)?;
        let identifier = parse_identifier(tokens, ptr)?;
        are_tokens(tokens, &[TokenType::OpenParen, TokenType::Void, TokenType::CloseParen, TokenType::OpenBrace], ptr)?;

        let mut function_body = vec![];
        while tokens[*ptr].token_type != TokenType::CloseBrace {
                function_body.push(parse_block_item(tokens, ptr)?);
        }
        *ptr += 1;

        Ok(AFunction { identifier, function_body })
}

// <block-item> ::= <statement> | <declaration>
fn parse_block_item(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<BlockItem, Error> {
        match tokens[*ptr].token_type {
                TokenType::Int => Ok(BlockItem::D(parse_declaration(tokens, ptr)?)),
                _ => Ok(BlockItem::S(parse_statement(tokens, ptr)?)),
        }
}

// <declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
fn parse_declaration(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<Declaration, Error> {
        assert_eq!(tokens[*ptr].token_type, TokenType::Int);
        *ptr += 1;

        let (start, len) = is_token(tokens, TokenType::Identifier, ptr)?;
        let id = AIdentifier { start, len };
        let mut init = None;

        if tokens[*ptr].token_type == TokenType::Equal {
                *ptr += 1;
                init = Some(parse_expression(tokens, ptr, 0)?);
        }

        is_token(tokens, TokenType::SemiColon, ptr)?;
        Ok(Declaration { id, init })
}

/* <statement> ::= "return" <exp> ";" | <exp> ; | ;
| "if" "(" <exp> ")" <statement> [ "else" <statement> ] */
fn parse_statement(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<AStatement, Error> {
        if is_token(tokens, TokenType::Return, ptr).is_ok() {
                let expr = parse_expression(tokens, ptr, 0)?;
                is_token(tokens, TokenType::SemiColon, ptr)?;
                Ok(AStatement::Return(expr))
        } else if let Ok(expr) = parse_expression(tokens, ptr, 0) {
                is_token(tokens, TokenType::SemiColon, ptr)?;
                Ok(AStatement::Expr(expr))
        } else if are_tokens(&tokens, &[TokenType::If, TokenType::OpenParen], ptr).is_ok() {
                let condition = parse_expression(tokens, ptr, 0)?;
                assert_eq!(tokens[*ptr].token_type, TokenType::CloseParen);
                *ptr += 1;

                let then = Box::new(parse_statement(tokens, ptr)?);

                let mut Else = None;
                if is_token(&tokens, TokenType::Else, ptr).is_ok() {
                        Else = Some(Box::new(parse_statement(tokens, ptr)?));
                }

                Ok(AStatement::I(IfStatement { condition, then, Else }))
        } else {
                is_token(tokens, TokenType::SemiColon, ptr)?;
                Ok(AStatement::Nul)
        }
}

pub static ASSIGNBINOP: [Binop; 12] = [
        Binop::AddAssign,
        Binop::SubtractAssign,
        Binop::MultiplyAssign,
        Binop::DivideAssign,
        Binop::RemainderAssign,
        Binop::LeftShiftAssign,
        Binop::RightShiftAssign,
        Binop::BitwiseAndAssign,
        Binop::LogicalAndAssign,
        Binop::BitwiseOrAssign,
        Binop::LogicalOrAssign,
        Binop::BitwiseXOrAssign,
];

// <exp> ::= <factor> | <exp> <binop> <exp> | <exp> "?" <exp> ":" <exp>
fn parse_expression(tokens: &mut Vec<Token>, ptr: &mut usize, min_precedence: usize) -> Result<AExpression, Error> {
        let mut left = AExpression::F(parse_factor(tokens, ptr)?);
        while let Some(operator) = parse_binary_operator(tokens, ptr) {
                let operator_precedence = binary_operator_precedence(operator);

                if operator_precedence < min_precedence {
                        *ptr -= 1;
                        break;
                }

                if operator == Binop::Equal {
                        let right = parse_expression(tokens, ptr, operator_precedence)?;
                        left = AExpression::Assignment(Box::new(left), Box::new(right));
                } else if ASSIGNBINOP.contains(&operator) {
                        let right = parse_expression(tokens, ptr, operator_precedence)?;
                        left = AExpression::OpAssignment(operator, Box::new(left), Box::new(right));
                } else if operator == Binop::Ternary {
                        let middle = parse_expression(tokens, ptr, 0)?;
                        assert_eq!(tokens[*ptr].token_type, TokenType::Colon);
                        *ptr += 1;

                        let right = parse_expression(tokens, ptr, operator_precedence)?;

                        left = AExpression::C(Conditional {
                                condition: Box::new(left),
                                True: Box::new(middle),
                                False: Box::new(right),
                        })
                } else {
                        let right = parse_expression(tokens, ptr, operator_precedence + 1)?;
                        left = AExpression::BinOp(operator, Box::new(left), Box::new(right));
                }
        }

        Ok(left)
}

// <factor> ::= <identifier> | <int> | "(" <exp> ")" | <unop> <factor> | <factor> <unop> (postfix in/decrement)
fn parse_factor(tokens: &mut Vec<Token>, ptr: &mut usize) -> Result<AFactor, Error> {
        let postfix_unop = |f: Token| {
                if let Some(incdec) = match f.token_type {
                        TokenType::DoubleMinus => Some(Unop::DecrementPost),
                        TokenType::DoublePlus => Some(Unop::IncrementPost),
                        _ => None,
                } {
                        Some(incdec)
                } else {
                        None
                }
        };

        if let Ok(identifier) = parse_identifier(tokens, ptr) {
                let mut temp = AFactor::Id(identifier);
                while let Some(incdec) = postfix_unop(tokens[*ptr]) {
                        *ptr += 1;
                        temp = AFactor::Unop(incdec, Box::new(temp));
                }

                return Ok(temp);
        }

        if let Ok(constant) = parse_constant(tokens, ptr) {
                let mut temp = AFactor::Constant(constant);

                while let Some(incdec) = postfix_unop(tokens[*ptr]) {
                        *ptr += 1;
                        temp = AFactor::Unop(incdec, Box::new(temp));
                }

                return Ok(temp);
        }

        if is_token(tokens, TokenType::OpenParen, ptr).is_ok() {
                if let Ok(expr) = parse_expression(tokens, ptr, 0) {
                        if is_token(tokens, TokenType::CloseParen, ptr).is_ok() {
                                let mut temp = AFactor::Expr(Box::new(expr));
                                while let Some(incdec) = postfix_unop(tokens[*ptr]) {
                                        *ptr += 1;
                                        temp = AFactor::Unop(incdec, Box::new(temp));
                                }

                                return Ok(temp);
                        }
                        *ptr -= 1;
                }
                *ptr -= 1;
        }

        if let Some(unop) = parse_unary_operator(tokens, ptr) {
                if let Ok(factor) = parse_factor(tokens, ptr) {
                        let mut temp = AFactor::Unop(unop, Box::new(factor));

                        while let Some(incdec) = postfix_unop(tokens[*ptr]) {
                                *ptr += 1;
                                temp = AFactor::Unop(incdec, Box::new(temp));
                        }
                        return Ok(temp);
                }
                *ptr -= 1;
        }

        Err(Error::InvalidFactorAt(tokens[*ptr]))
}

// <unop> ::= "-" | "~" | "!" | ++ (post) | -- (post)
fn parse_unary_operator(tokens: &[Token], ptr: &mut usize) -> Option<Unop> {
        if let Some(unop) = match tokens[*ptr].token_type {
                TokenType::Minus => Some(Unop::Negate),
                TokenType::Tilde => Some(Unop::Complement),
                TokenType::Not => Some(Unop::Not),
                TokenType::DoubleMinus => Some(Unop::DecrementPost),
                TokenType::DoublePlus => Some(Unop::IncrementPost),
                _ => None,
        } {
                *ptr += 1;
                Some(unop)
        } else {
                None
        }
}

/*
<binop> ::= "-" | "+" | "*" | "/" | "%" | "<<" | ">>" | "&" | "|" | | "^"
| "&&" | "||" | "==" | "!=" | "<" | "<=" | ">" | ">="
| += | -= | *= | /= | %= | &= | ^= | <<= | >>= | ?
*/
fn parse_binary_operator(tokens: &[Token], ptr: &mut usize) -> Option<Binop> {
        if let Some(binop) = match tokens[*ptr].token_type {
                TokenType::Minus => Some(Binop::Subtract),
                TokenType::Plus => Some(Binop::Add),
                TokenType::Asterisk => Some(Binop::Multiply),
                TokenType::ForwardSlash => Some(Binop::Divide),
                TokenType::Percent => Some(Binop::Remainder),
                TokenType::BitwiseAnd => Some(Binop::BitwiseAnd),
                TokenType::LogicalAnd => Some(Binop::LogicalAnd),
                TokenType::BitwiseOr => Some(Binop::BitwiseOr),
                TokenType::LogicalOr => Some(Binop::LogicalOr),
                TokenType::BitwiseXOr => Some(Binop::BitwiseXOr),
                TokenType::LeftShift => Some(Binop::LeftShift),
                TokenType::RightShift => Some(Binop::RightShift),
                TokenType::LessThan => Some(Binop::LessThan),
                TokenType::MoreThan => Some(Binop::MoreThan),
                TokenType::LessThanOrEqual => Some(Binop::LessThanOrEqual),
                TokenType::MoreThanOrEqual => Some(Binop::MoreThanOrEqual),
                TokenType::EqualTo => Some(Binop::EqualTo),
                TokenType::NotEqualTo => Some(Binop::NotEqualTo),
                TokenType::Equal => Some(Binop::Equal),
                TokenType::AddAssign => Some(Binop::AddAssign),
                TokenType::SubtractAssign => Some(Binop::SubtractAssign),
                TokenType::MultiplyAssign => Some(Binop::MultiplyAssign),
                TokenType::DivideAssign => Some(Binop::DivideAssign),
                TokenType::RemainderAssign => Some(Binop::RemainderAssign),
                TokenType::LeftShiftAssign => Some(Binop::LeftShiftAssign),
                TokenType::RightShiftAssign => Some(Binop::RightShiftAssign),
                TokenType::BitwiseAndAssign => Some(Binop::BitwiseAndAssign),
                TokenType::LogicalAndAssign => Some(Binop::LogicalAndAssign),
                TokenType::BitwiseOrAssign => Some(Binop::BitwiseOrAssign),
                TokenType::LogicalOrAssign => Some(Binop::LogicalOrAssign),
                TokenType::BitwiseXOrAssign => Some(Binop::BitwiseXOrAssign),
                TokenType::Ternary => Some(Binop::Ternary),
                _ => None,
        } {
                *ptr += 1;
                Some(binop)
        } else {
                None
        }
}

// <identifier> ::= ? An identifier token ?
fn parse_identifier(tokens: &[Token], ptr: &mut usize) -> Result<AIdentifier, Error> {
        let (start, len) = is_token(tokens, TokenType::Identifier, ptr)?;

        Ok(AIdentifier { start, len })
}

// <int> ::= ? A constant token ?
fn parse_constant(tokens: &[Token], ptr: &mut usize) -> Result<AConstant, Error> {
        let (start, len) = is_token(tokens, TokenType::Constant, ptr)?;

        Ok(AConstant { start, len })
}

fn is_token(tokens: &[Token], wanted_token_type: TokenType, ptr: &mut usize) -> Result<(usize, usize), Error> {
        let Some(&Token { token_type, len, start }) = tokens.get(*ptr) else {
                return Err(Error::NotEnoughTokens);
        };

        if token_type == wanted_token_type {
                *ptr += 1;
                return Ok((start, len));
        }

        Err(Error::InvalidTokenAt(tokens[*ptr], wanted_token_type))
}

fn are_tokens(tokens: &[Token], wanted_token_type: &[TokenType], ptr: &mut usize) -> Result<(), Error> {
        for (idx, &i) in wanted_token_type.iter().enumerate() {
                if tokens[*ptr + idx].token_type != i {
                        return Err(Error::InvalidTokenAt(tokens[*ptr + idx], i));
                }
        }
        *ptr += wanted_token_type.len();
        Ok(())
}

fn binary_operator_precedence(operator: Binop) -> usize {
        match operator {
                Binop::Multiply | Binop::Divide | Binop::Remainder => 50,
                Binop::Add | Binop::Subtract => 45,
                Binop::LeftShift | Binop::RightShift => 37,
                Binop::LessThan | Binop::LessThanOrEqual | Binop::MoreThan | Binop::MoreThanOrEqual => 35,
                Binop::EqualTo | Binop::NotEqualTo => 30,
                Binop::BitwiseAnd => 20,
                Binop::BitwiseXOr => 17,
                Binop::BitwiseOr => 15,
                Binop::LogicalAnd => 10,
                Binop::LogicalOr => 5,
                Binop::Ternary => 3,
                Binop::Equal
                | Binop::AddAssign
                | Binop::SubtractAssign
                | Binop::MultiplyAssign
                | Binop::DivideAssign
                | Binop::RemainderAssign
                | Binop::LeftShiftAssign
                | Binop::RightShiftAssign
                | Binop::BitwiseAndAssign
                | Binop::LogicalAndAssign
                | Binop::BitwiseOrAssign
                | Binop::LogicalOrAssign
                | Binop::BitwiseXOrAssign => 1,
        }
}
