use crate::parse::nodes::BlockItem::D;
use crate::parse::nodes::BlockItem::S;
use crate::{
        parse::{
                nodes::{AExpression, AFactor, AIdentifier, AProgram, AStatement, Declaration},
                Parsed,
        },
        Program, State,
};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
        #[error("Variable {0} was declared twice, second at {1}")]
        DeclaredTwice(String, usize),
        #[error("Invalid left side of assignment\n{0}")]
        InvalidLValue(AExpression),
        #[error("Variable {0} was not declared, at {1}")]
        UndeclaredVariable(String, usize),
}

fn resolve_declaration<'b, 'a: 'b>(
        code: &'a [u8],
        declaration: &Declaration,
        mut identifier_set: HashSet<(&'b [u8], usize)>,
        scope: usize,
) -> Result<HashSet<(&'b [u8], usize)>, Error> {
        let AIdentifier { start, len } = declaration.id;
        let name = &code[start..start + len];
        let entry = (name, scope);

        if identifier_set.contains(&entry) {
                return Err(Error::DeclaredTwice(String::from_utf8(entry.0.to_vec()).unwrap(), start));
        }

        identifier_set.insert(entry);
        if let Some(expr) = &declaration.init {
                _ = Some(resolve_exp(code, expr, &mut identifier_set, scope)?);
        }

        Ok(identifier_set)
}

fn resolve_statement<'b, 'a: 'b>(
        code: &'a [u8],
        statement: &AStatement,
        mut identifier_set: HashSet<(&'b [u8], usize)>,
        scope: usize,
) -> Result<HashSet<(&'b [u8], usize)>, Error> {
        match statement {
                AStatement::Return(aexpression) => {
                        _ = resolve_exp(code, aexpression, &mut identifier_set, scope)?;
                }
                AStatement::Expr(aexpression) => {
                        _ = resolve_exp(code, aexpression, &mut identifier_set, scope)?;
                }
                AStatement::Nul => {}
        }

        Ok(identifier_set)
}

fn resolve_exp(code: &[u8], expr: &AExpression, identifier_set: &mut HashSet<(&[u8], usize)>, scope: usize) -> Result<(), Error> {
        match expr {
                AExpression::Assignment(lval, rval) => match **lval {
                        AExpression::F(AFactor::Id(id)) => {
                                let AIdentifier { start, len } = id;
                                let name = &code[start..start + len];

                                let _ = resolve_exp(code, rval, identifier_set, scope)?;

                                if !identifier_set.contains(&(name, scope)) {
                                        let name = String::from_utf8(name.to_vec()).unwrap();
                                        return Err(Error::UndeclaredVariable(name, start));
                                }
                        }
                        _ => return Err(Error::InvalidLValue(expr.clone())),
                },
                AExpression::BinOp(_bin_op, left, right) => {
                        _ = resolve_exp(code, &*left, identifier_set, scope)?;
                        _ = resolve_exp(code, &*right, identifier_set, scope)?;
                }
                AExpression::F(afactor) => resolve_factor(code, afactor, identifier_set, scope)?,
        }
        Ok(())
}

fn resolve_factor(code: &[u8], factor: &AFactor, identifier_set: &mut HashSet<(&[u8], usize)>, scope: usize) -> Result<(), Error> {
        match factor {
                AFactor::Id(id) => {
                        let AIdentifier { start, len } = id;
                        let name = &code[*start..*start + *len];

                        if !identifier_set.contains(&(name, scope)) {
                                let name = String::from_utf8(name.to_vec()).unwrap();
                                return Err(Error::UndeclaredVariable(name, *start));
                        }
                }
                AFactor::Constant(_) => {}
                AFactor::Unop(_unop, afactor) => resolve_factor(code, &*afactor, identifier_set, scope)?,
                AFactor::Expr(expr) => resolve_exp(code, &*expr, identifier_set, scope)?,
        }

        Ok(())
}

#[derive(Debug, Clone)]
pub struct SemanticallyAnalyzed {
        pub code: Vec<u8>,
        pub program: AProgram,
}
impl State for SemanticallyAnalyzed {}

pub fn analyze(value: Program<Parsed>) -> Result<Program<SemanticallyAnalyzed>, Error> {
        let mut identifier_set = HashSet::new();
        let program = value.state.program;
        let code = value.state.code;

        for i in program.functions.function_body.iter() {
                match i {
                        D(declaration) => identifier_set = resolve_declaration(&code, declaration, identifier_set, 0)?,
                        S(astatement) => identifier_set = resolve_statement(&code, astatement, identifier_set, 0)?,
                }
        }

        Ok(Program {
                operation: value.operation,
                state: SemanticallyAnalyzed { code, program },
        })
}
