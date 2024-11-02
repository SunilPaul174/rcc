use std::collections::HashMap;

use crate::{
        parse::{
                nodes::{AExpression, AFactor, AIdentifier, AProgram, AStatement, BlockItem, Conditional, Declaration, IfStatement, Unop},
                Parsed,
        },
        tactile::Identifier,
        Program, State,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
        #[error("Variable {0} was declared twice, second at {1}")]
        DeclaredTwice(String, usize),
        #[error("Invalid left side of assignment expr: \n{0}")]
        InvalidLValueExpr(AExpression),
        #[error("Variable {0} was not declared, at {1}")]
        UndeclaredVariable(String, usize),
        #[error("Invalid left side of assignment factor: \n{0:?}")]
        InvalidLValueFactor(AFactor),
}

#[derive(Debug, Clone)]
pub struct SemanticallyAnalyzed {
        pub code: Vec<u8>,
        pub program: AProgram,
}
impl State for SemanticallyAnalyzed {}

pub fn analyze(value: Program<Parsed>) -> Result<Program<SemanticallyAnalyzed>, Error> {
        let code = value.state.code;
        let program = value.state.program;

        let mut global_max_identifier = 0;
        let scope = 0;

        let mut variable_map = HashMap::new();

        for i in program.function.function_body.iter() {
                match i {
                        BlockItem::D(declaration) => resolve_declaration(&code, declaration, &mut variable_map, &mut global_max_identifier, scope)?,
                        BlockItem::S(astatement) => resolve_statement(&code, astatement, &mut variable_map, scope)?,
                }
        }

        Ok(Program {
                operation: value.operation,
                state: SemanticallyAnalyzed { code, program },
        })
}

fn resolve_declaration<'b, 'a: 'b>(
        code: &'a [u8],
        declaration: &Declaration,
        variable_map: &mut HashMap<(&'b [u8], usize), Identifier>,
        global_max_identifier: &mut usize,
        scope: usize,
) -> Result<(), Error> {
        let AIdentifier { start, len } = declaration.id;
        let name = &code[start..start + len];
        if variable_map.get(&(name, scope)).is_some() {
                return Err(Error::DeclaredTwice(String::from_utf8(name.to_vec()).unwrap(), start));
        }
        let temp = *global_max_identifier;
        *global_max_identifier += 1;

        variable_map.entry((name, scope)).insert_entry(Identifier(temp));

        if let Some(extract) = &declaration.init {
                () = resolve_exp(code, extract, variable_map, scope)?;
        }

        Ok(())
}

fn resolve_statement(code: &[u8], statement: &AStatement, variable_map: &mut HashMap<(&[u8], usize), Identifier>, scope: usize) -> Result<(), Error> {
        match statement {
                AStatement::Return(expr) => resolve_exp(code, expr, variable_map, scope),
                AStatement::Expr(expr) => resolve_exp(code, expr, variable_map, scope),
                AStatement::I(If) => {
                        let IfStatement { condition, then, Else } = If;
                        resolve_exp(code, condition, variable_map, scope)?;
                        resolve_statement(code, then, variable_map, scope)?;
                        if let Some(Else) = Else {
                                resolve_statement(code, Else, variable_map, scope)?;
                        }

                        Ok(())
                }
                AStatement::Nul => Ok(()),
        }
}

fn resolve_exp(code: &[u8], expr: &AExpression, variable_map: &mut HashMap<(&[u8], usize), Identifier>, scope: usize) -> Result<(), Error> {
        match expr {
                AExpression::F(afactor) => match afactor {
                        AFactor::Expr(aexpression) => resolve_exp(code, aexpression, variable_map, scope),
                        AFactor::Id(aidentifier) => variable_exists(code, aidentifier, variable_map, scope),
                        AFactor::Unop(unop, afactor) => is_valid_lvalue_unop(code, *unop, *afactor.clone(), variable_map, scope),
                        AFactor::Constant(_) => Ok(()),
                },
                AExpression::Assignment(left, right) => {
                        is_valid_lvalue_assignment(code, left, variable_map, scope)?;
                        resolve_exp(code, right, variable_map, scope)
                }
                AExpression::BinOp(_, left, right) => {
                        resolve_exp(code, left, variable_map, scope)?;
                        resolve_exp(code, right, variable_map, scope)
                }
                AExpression::C(Conditional { condition, True, False }) => {
                        resolve_exp(code, condition, variable_map, scope)?;
                        resolve_exp(code, True, variable_map, scope)?;
                        resolve_exp(code, False, variable_map, scope)
                }
                AExpression::OpAssignment(_binop, left, right) => {
                        is_valid_lvalue_assignment(code, left, variable_map, scope)?;
                        resolve_exp(code, right, variable_map, scope)
                }
        }
}

fn is_valid_lvalue_assignment(code: &[u8], left: &AExpression, variable_map: &mut HashMap<(&[u8], usize), Identifier>, scope: usize) -> Result<(), Error> {
        match left {
                AExpression::F(afactor) => match afactor {
                        AFactor::Expr(expr) => is_valid_lvalue_assignment(code, expr, variable_map, scope),
                        AFactor::Id(aidentifier) => variable_exists(code, aidentifier, variable_map, scope),
                        AFactor::Constant(..) | AFactor::Unop(..) => return Err(Error::InvalidLValueExpr(left.clone())),
                },
                AExpression::Assignment(left, right) => {
                        resolve_exp(code, left, variable_map, scope)?;
                        resolve_exp(code, right, variable_map, scope)
                }
                AExpression::C(_) | AExpression::BinOp(..) | AExpression::OpAssignment(..) => return Err(Error::InvalidLValueExpr(left.clone())),
        }
}

fn is_valid_lvalue_unop(code: &[u8], unop: Unop, factor: AFactor, variable_map: &mut HashMap<(&[u8], usize), Identifier>, scope: usize) -> Result<(), Error> {
        match factor.clone() {
                AFactor::Constant(_) => match unop {
                        Unop::Negate | Unop::Complement | Unop::Not => Ok(()),
                        Unop::IncrementPre | Unop::IncrementPost | Unop::DecrementPre | Unop::DecrementPost => Err(Error::InvalidLValueFactor(factor)),
                },
                AFactor::Unop(innerunop, afactor) => {
                        match unop {
                                Unop::Negate | Unop::Complement | Unop::Not => {}
                                Unop::IncrementPre | Unop::IncrementPost | Unop::DecrementPre | Unop::DecrementPost => return Err(Error::InvalidLValueFactor(factor)),
                        }
                        is_valid_lvalue_unop(code, innerunop, *afactor, variable_map, scope)?;

                        Ok(())
                }
                AFactor::Expr(aexpression) => {
                        resolve_exp(code, &aexpression, variable_map, scope)?;
                        match *aexpression {
                                AExpression::F(afactor) => is_valid_lvalue_unop(code, unop, afactor, variable_map, scope),
                                AExpression::Assignment(..) | AExpression::C(_) | AExpression::OpAssignment(..) => Err(Error::InvalidLValueExpr(*aexpression)),
                                AExpression::BinOp(_binop, left, right) => {
                                        match unop {
                                                Unop::Negate | Unop::Complement | Unop::Not => {}
                                                Unop::IncrementPre | Unop::IncrementPost | Unop::DecrementPre | Unop::DecrementPost => {
                                                        return Err(Error::InvalidLValueFactor(factor))
                                                }
                                        }
                                        resolve_exp(code, &left, variable_map, scope)?;
                                        resolve_exp(code, &right, variable_map, scope)
                                }
                        }
                }
                AFactor::Id(aidentifier) => variable_exists(code, &aidentifier, variable_map, scope),
        }
}

fn variable_exists(code: &[u8], aidentifier: &AIdentifier, variable_map: &mut HashMap<(&[u8], usize), Identifier>, scope: usize) -> Result<(), Error> {
        let &AIdentifier { start, len } = aidentifier;
        let name = &code[start..start + len];

        if !variable_map.get(&(name, scope)).is_some() {
                return Err(Error::UndeclaredVariable(String::from_utf8(name.to_vec()).unwrap(), start));
        }

        Ok(())
}
