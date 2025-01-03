use nodes::{
        ABlock, AConstant, AExpression, AFactor, AIdentifier, AProgram, AStatement, Binop, BlockItem, BreakType,
        Conditional, Declaration, For, ForInit, FunctionDeclaration, IfStatement, LoopSwitchOrNone, ParseLabel, Switch,
        Unop, VariableDeclaration,
};
use thiserror::Error;

use crate::{
        lex::{
                tokentype::{Token, TokenType},
                Lexed,
        },
        Program, State,
};

pub mod nodes;

#[derive(Debug, Clone)]
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
        #[error("Trailing comma in function declaration")]
        TrailingCommaInParamList,
        #[error("Break outside loop")]
        BreakOutsideLoop,
}

// <program> ::= { <function-declaration> }
pub fn parse_program(mut program: Program<Lexed>) -> Result<Program<Parsed>, Error> {
        let mut ptr = 0;

        let mut functions = vec![];

        while let Some(function) = parse_function_declaration(&mut program.state.tokens, &mut ptr)? {
                functions.push(function);
        }

        if ptr < program.state.tokens.len() {
                return Err(Error::TooManyTokens);
        }

        Ok(Program {
                state: Parsed {
                        code: program.state.code,
                        program: AProgram { functions },
                },
                operation: program.operation,
                obj: program.obj,
        })
}

// <block> ::= "{" { <block-item> } "}"
fn parse_block(tokens: &[Token], ptr: &mut usize, curr_state: LoopSwitchOrNone) -> Result<ABlock, Error> {
        is_token(tokens, TokenType::OpenBrace, ptr)?;
        let mut block = vec![];
        while tokens[*ptr].token_type != TokenType::CloseBrace {
                block.push(parse_block_item(tokens, ptr, curr_state)?);
        }
        *ptr += 1;
        Ok(ABlock(block))
}

// <block-item> ::= <statement> | <declaration>
fn parse_block_item(tokens: &[Token], ptr: &mut usize, curr_state: LoopSwitchOrNone) -> Result<BlockItem, Error> {
        match tokens[*ptr].token_type {
                TokenType::Int => Ok(BlockItem::D(parse_declaration(tokens, ptr)?)),
                _ => Ok(BlockItem::S(parse_statement(tokens, ptr, curr_state)?)),
        }
}

// <declaration> ::= <variable-declaration> | <function-declaration>
fn parse_declaration(tokens: &[Token], ptr: &mut usize) -> Result<Declaration, Error> {
        if [TokenType::Equal, TokenType::SemiColon].contains(&tokens[*ptr + 2].token_type) {
                Ok(Declaration::V(parse_variable_declaration(tokens, ptr)?))
        } else {
                Ok(Declaration::F(parse_function_declaration(tokens, ptr)?.unwrap()))
        }
}

// <function-declaration> ::= "int" <identifier> "(" <param-list> ")" ( <block> | ";")
fn parse_function_declaration(tokens: &[Token], ptr: &mut usize) -> Result<Option<FunctionDeclaration>, Error> {
        if tokens.len() == *ptr {
                return Ok(None);
        }

        is_token(tokens, TokenType::Int, ptr)?;

        let name = parse_identifier(tokens, ptr)?;

        is_token(tokens, TokenType::OpenParen, ptr)?;

        let params = parse_param_list(tokens, ptr)?;

        is_token(tokens, TokenType::CloseParen, ptr)?;

        let mut body = None;
        if tokens[*ptr].token_type == TokenType::OpenBrace {
                body = Some(parse_block(tokens, ptr, LoopSwitchOrNone::Neither)?);
        } else {
                is_token(tokens, TokenType::SemiColon, ptr)?;
        }

        Ok(Some(FunctionDeclaration { name, params, body }))
}

// <param-list> ::= "void" | "int" <identifier> { "," "int" <identifier> }
fn parse_param_list(tokens: &[Token], ptr: &mut usize) -> Result<Option<Vec<AIdentifier>>, Error> {
        if is_token(tokens, TokenType::Void, ptr).is_ok() {
                return Ok(None);
        }

        let mut parameters = vec![];

        loop {
                if is_token(tokens, TokenType::Int, ptr).is_ok() {
                        parameters.push(parse_identifier(tokens, ptr)?);
                } else if is_token(tokens, TokenType::Comma, ptr).is_ok() {
                        continue;
                } else {
                        break;
                }
        }

        if tokens[*ptr - 1].token_type == TokenType::Comma {
                return Err(Error::TrailingCommaInParamList);
        }

        Ok(Some(parameters))
}

// <variable-declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
fn parse_variable_declaration(tokens: &[Token], ptr: &mut usize) -> Result<VariableDeclaration, Error> {
        is_token(tokens, TokenType::Int, ptr)?;

        let (start, len) = is_token(tokens, TokenType::Identifier, ptr)?;
        let id = AIdentifier { start, len };
        let mut init = None;

        if tokens[*ptr].token_type == TokenType::Equal {
                *ptr += 1;
                init = Some(parse_expression(tokens, ptr, 0)?);
        }

        is_token(tokens, TokenType::SemiColon, ptr)?;
        Ok(VariableDeclaration { id, init })
}

/* <statement> ::= "return" <exp> ";"
| <exp> ";"
| "if" "(" <exp> ")" <statement> [ "else" <statement> ]
| <block>
| "break" ";"
| "continue" ";"
| "while" "(" <exp> ")" <statement>
| "do" <statement> "while" "(" <exp> ")" ";"
| "for" "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
| ";"
| "switch" (aexpression) "{" [ { "case" ":" <statement> } ] [ "default" ":" <statement> ] "}"
*/
fn parse_statement(tokens: &[Token], ptr: &mut usize, curr_state: LoopSwitchOrNone) -> Result<AStatement, Error> {
        // println!("curr_state: {:?}, curr_token: {:?}", curr_state, tokens[*ptr]);
        if is_token(tokens, TokenType::Return, ptr).is_ok() {
                let expr = parse_expression(tokens, ptr, 0)?;
                is_token(tokens, TokenType::SemiColon, ptr)?;
                Ok(AStatement::Return(expr))
        } else if let Ok(expr) = parse_expression(tokens, ptr, 0) {
                is_token(tokens, TokenType::SemiColon, ptr)?;
                Ok(AStatement::Expr(expr))
        } else if are_tokens(tokens, &[TokenType::If, TokenType::OpenParen], ptr).is_ok() {
                let condition = parse_expression(tokens, ptr, 0)?;
                is_token(tokens, TokenType::CloseParen, ptr)?;

                let then = Box::new(parse_statement(tokens, ptr, curr_state)?);

                let mut Else = None;
                if is_token(tokens, TokenType::Else, ptr).is_ok() {
                        Else = Some(Box::new(parse_statement(tokens, ptr, curr_state)?));
                }

                Ok(AStatement::I(IfStatement { condition, then, Else }))
        } else if is_token(tokens, TokenType::Switch, ptr).is_ok() {
                is_token(tokens, TokenType::OpenParen, ptr)?;
                let aexpression = parse_expression(tokens, ptr, 0)?;
                are_tokens(tokens, &[TokenType::CloseParen, TokenType::OpenBrace], ptr)?;

                let mut cases = vec![];
                let mut default = None;

                loop {
                        if is_token(tokens, TokenType::Case, ptr).is_ok() {
                                let constant = parse_constant(tokens, ptr)?;
                                is_token(tokens, TokenType::Colon, ptr)?;
                                let mut statements = vec![];
                                while let Ok(statement) = parse_statement(tokens, ptr, LoopSwitchOrNone::Switch) {
                                        statements.push(statement);
                                }
                                cases.push((constant, statements));
                                continue;
                        }

                        let cond = are_tokens(tokens, &[TokenType::Default, TokenType::Colon], ptr);

                        if cond.is_ok() && default.is_none() {
                                default = Some(Box::new(parse_statement(tokens, ptr, LoopSwitchOrNone::Switch)?));
                        } else if cond.is_ok() && default.is_some() {
                                return Err(Error::InvalidTokenAt(tokens[*ptr], TokenType::Default));
                        } else {
                                break;
                        }
                }

                is_token(tokens, TokenType::CloseBrace, ptr)?;

                Ok(AStatement::S(Switch {
                        value: aexpression,
                        cases,
                        default,
                        label: ParseLabel(0),
                }))
        } else if let Ok(block) = parse_block(tokens, ptr, curr_state) {
                Ok(AStatement::Compound(block))
        } else if are_tokens(tokens, &[TokenType::Break, TokenType::SemiColon], ptr).is_ok() {
                match curr_state {
                        LoopSwitchOrNone::Loop => Ok(AStatement::Break(ParseLabel(0), BreakType::Loop)),
                        LoopSwitchOrNone::Switch => Ok(AStatement::Break(ParseLabel(0), BreakType::Switch)),
                        LoopSwitchOrNone::Neither => Err(Error::BreakOutsideLoop),
                }
        } else if are_tokens(tokens, &[TokenType::Continue, TokenType::SemiColon], ptr).is_ok() {
                Ok(AStatement::Continue(ParseLabel(0)))
        } else if are_tokens(tokens, &[TokenType::While, TokenType::OpenParen], ptr).is_ok() {
                let expr = parse_expression(tokens, ptr, 0)?;
                is_token(tokens, TokenType::CloseParen, ptr)?;
                let statement = parse_statement(tokens, ptr, LoopSwitchOrNone::Loop)?;

                Ok(AStatement::While(expr, Box::new(statement), ParseLabel(0)))
        } else if is_token(tokens, TokenType::Do, ptr).is_ok() {
                let statement = parse_statement(tokens, ptr, LoopSwitchOrNone::Loop)?;
                are_tokens(tokens, &[TokenType::While, TokenType::OpenParen], ptr)?;
                let expr = parse_expression(tokens, ptr, 0)?;
                are_tokens(tokens, &[TokenType::CloseParen, TokenType::SemiColon], ptr)?;

                Ok(AStatement::DoWhile(Box::new(statement), expr, ParseLabel(0)))
        } else if are_tokens(tokens, &[TokenType::For, TokenType::OpenParen], ptr).is_ok() {
                let init = parse_for_init(tokens, ptr)?;

                let (mut post, mut condition) = (None, None);
                if let Ok(expr) = parse_expression(tokens, ptr, 0) {
                        condition = Some(expr);
                }
                is_token(tokens, TokenType::SemiColon, ptr)?;
                if let Ok(expr) = parse_expression(tokens, ptr, 0) {
                        post = Some(expr);
                }
                is_token(tokens, TokenType::CloseParen, ptr)?;
                let body = parse_statement(tokens, ptr, LoopSwitchOrNone::Loop)?;

                Ok(AStatement::F(
                        Box::new(For {
                                init,
                                condition,
                                post,
                                body,
                        }),
                        ParseLabel(0),
                ))
        } else {
                is_token(tokens, TokenType::SemiColon, ptr)?;
                Ok(AStatement::Nul)
        }
}

// <for-init> ::= <variable_declaration> | [ <exp> ] ";"
fn parse_for_init(tokens: &[Token], ptr: &mut usize) -> Result<ForInit, Error> {
        if let Ok(variable) = parse_variable_declaration(tokens, ptr) {
                return Ok(ForInit::D(variable));
        }

        let mut expression = None;
        if let Ok(expr) = parse_expression(tokens, ptr, 0) {
                expression = Some(expr);
        }
        is_token(tokens, TokenType::SemiColon, ptr)?;

        Ok(ForInit::E(expression))
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
fn parse_expression(tokens: &[Token], ptr: &mut usize, min_precedence: usize) -> Result<AExpression, Error> {
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
                        is_token(tokens, TokenType::Colon, ptr)?;

                        let right = parse_expression(tokens, ptr, operator_precedence)?;

                        left = AExpression::C(Conditional {
                                condition: Box::new(left),
                                True: Box::new(middle),
                                False: Box::new(right),
                        });
                } else {
                        let right = parse_expression(tokens, ptr, operator_precedence + 1)?;
                        left = AExpression::BinOp(operator, Box::new(left), Box::new(right));
                }
        }

        Ok(left)
}

// <argument-list> ::= <exp> { "," <exp> }
fn parse_call_list(tokens: &[Token], ptr: &mut usize) -> Result<Option<Vec<AExpression>>, Error> {
        let mut params = vec![];

        while let Ok(expr) = parse_expression(tokens, ptr, 0) {
                params.push(expr);

                if is_token(tokens, TokenType::Comma, ptr).is_err() {
                        break;
                }
        }

        if tokens[*ptr - 1].token_type == TokenType::Comma {
                return Err(Error::TrailingCommaInParamList);
        }

        if params.is_empty() {
                Ok(None)
        } else {
                Ok(Some(params))
        }
}

// <factor> ::= <identifier> | <int> | "(" <exp> ")" | <unop> <factor> | <factor> <unop> (postfix in/decrement)
// <identifier> "(" [ <argument-list> ] ")"
fn parse_factor(tokens: &[Token], ptr: &mut usize) -> Result<AFactor, Error> {
        let postfix_unop = |f: Token| match f.token_type {
                TokenType::DoubleMinus => Some(Unop::DecrementPost),
                TokenType::DoublePlus => Some(Unop::IncrementPost),
                _ => None,
        };

        if let Ok(identifier) = parse_identifier(tokens, ptr) {
                let mut temp = AFactor::Id(identifier);

                if is_token(tokens, TokenType::OpenParen, ptr).is_ok() {
                        let list = parse_call_list(tokens, ptr)?;
                        temp = AFactor::Expr(Box::new(AExpression::FunctionCall(identifier, list)));
                        is_token(tokens, TokenType::CloseParen, ptr)?;
                }

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
        assert!(!wanted_token_type.is_empty());
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
