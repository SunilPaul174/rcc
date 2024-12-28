use std::collections::HashMap;

use crate::parse::nodes::{
        ABlock, AExpression, AFactor, AIdentifier, AProgram, AStatement, BlockItem, Conditional, Declaration, For,
        ForInit, FunctionDeclaration, IfStatement, Switch, VariableDeclaration,
};

use super::Error;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct FuncType {
        param_count: usize,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Type {
        Int,
        // have we type checked the definition of the function yet?
        Func(FuncType, bool),
}

pub fn type_check(program: &mut AProgram, code: &[u8]) -> Result<(), Error> {
        let mut symbols = HashMap::new();

        for i in &program.functions {
                check_function_declaration(i, &mut symbols, code, 0)?;
        }

        Ok(())
}

fn check_variable_declaration<'b, 'a: 'b>(
        decl: &VariableDeclaration,
        symbols: &mut HashMap<(&'b [u8], usize), Type>,
        code: &'a [u8],
        scope: usize,
) -> Result<(), Error> {
        // we already checked for uniqueness of variable names, so don't bother doing anything again obvi
        symbols.entry((name(code, decl.id), scope)).insert_entry(Type::Int);

        if let Some(expr) = &decl.init {
                check_expr(expr, symbols, code, scope)?;
        }

        Ok(())
}

fn check_expr<'b, 'a: 'b>(
        expr: &AExpression,
        symbols: &mut HashMap<(&'b [u8], usize), Type>,
        code: &'a [u8],
        scope: usize,
) -> Result<(), Error> {
        match expr {
                AExpression::F(afactor) => check_factor(afactor, symbols, code, scope),
                AExpression::BinOp(_, left, right)
                | AExpression::Assignment(left, right)
                | AExpression::OpAssignment(_, left, right) => {
                        check_expr(left, symbols, code, scope)?;
                        check_expr(right, symbols, code, scope)?;
                        Ok(())
                }
                AExpression::C(Conditional { condition, True, False }) => {
                        check_expr(condition, symbols, code, scope)?;
                        check_expr(True, symbols, code, scope)?;
                        check_expr(False, symbols, code, scope)?;
                        Ok(())
                }
                AExpression::FunctionCall(aidentifier, vec) => {
                        let name = name(code, *aidentifier);
                        let param_count = if let Some(params) = vec { params.len() } else { 0 };

                        let Ok(func_type) = symbol_exists(code, *aidentifier, symbols, scope) else {
                                return Err(Error::UndeclaredIdentifier(
                                        String::from_utf8(name.to_vec()).unwrap(),
                                        aidentifier.start,
                                ));
                        };

                        match func_type {
                                Type::Int => Err(Error::WrongType(
                                        String::from_utf8(name.to_vec()).unwrap(),
                                        Type::Int,
                                        Type::Func(FuncType { param_count }, true),
                                )),
                                Type::Func(func_type, defined) => {
                                        if func_type.param_count != param_count {
                                                return Err(Error::WrongType(
                                                        String::from_utf8(name.to_vec()).unwrap(),
                                                        Type::Func(func_type, defined),
                                                        Type::Func(FuncType { param_count }, true),
                                                ));
                                        }

                                        if let Some(params) = vec {
                                                for i in params {
                                                        check_expr(i, symbols, code, scope)?;
                                                }
                                        }

                                        Ok(())
                                }
                        }
                }
        }
}

fn check_factor<'b, 'a: 'b>(
        afactor: &AFactor,
        symbols: &mut HashMap<(&'b [u8], usize), Type>,
        code: &'a [u8],
        scope: usize,
) -> Result<(), Error> {
        match afactor {
                AFactor::Constant(_) => Ok(()),
                AFactor::Unop(_, afactor) => check_factor(afactor, symbols, code, scope),
                AFactor::Expr(aexpression) => check_expr(aexpression, symbols, code, scope),
                AFactor::Id(aidentifier) => {
                        let Ok(id_type) = symbol_exists(code, *aidentifier, symbols, scope) else {
                                return Err(Error::UndeclaredIdentifier(
                                        String::from_utf8(name(code, *aidentifier).to_vec()).unwrap(),
                                        aidentifier.start,
                                ));
                        };
                        if id_type != Type::Int {
                                return Err(Error::WrongType(
                                        String::from_utf8(name(code, *aidentifier).to_vec()).unwrap(),
                                        id_type,
                                        Type::Int,
                                ));
                        }
                        Ok(())
                }
        }
}

fn check_function_declaration<'b, 'a: 'b>(
        decl: &FunctionDeclaration,
        symbols: &mut HashMap<(&'b [u8], usize), Type>,
        code: &'a [u8],
        scope: usize,
) -> Result<(), Error> {
        let defined = decl.body.is_some();

        let func_type = FuncType {
                param_count: match &decl.params {
                        Some(list) => list.len(),
                        _ => 0,
                },
        };

        let mut prev_defined = false;

        if let Ok(Type::Func(entry_func_type, entry_defined)) = symbol_exists(code, decl.name, symbols, scope) {
                if func_type != entry_func_type {
                        return Err(Error::IncompatibleFunctionDeclarations(func_type, entry_func_type));
                }

                prev_defined = entry_defined;
                if defined && entry_defined {
                        return Err(Error::FunctionDefinedMoreThanOnce(decl.name));
                }
        }

        symbols.entry((name(code, decl.name), scope))
                .insert_entry(Type::Func(func_type, prev_defined | defined));
        symbols.entry((name(code, decl.name), 0))
                .insert_entry(Type::Func(func_type, prev_defined | defined));

        if let Some(body) = &decl.body {
                if let Some(vec) = &decl.params {
                        for param in vec {
                                symbols.entry((name(code, *param), scope + 1)).insert_entry(Type::Int);
                        }
                }

                check_block(body, symbols, code, scope + 1)?;
        }

        symbols.retain(|(_, f), _| *f < (scope + 1));

        Ok(())
}

fn check_block<'b, 'a: 'b>(
        block: &ABlock,
        symbols: &mut HashMap<(&'b [u8], usize), Type>,
        code: &'a [u8],
        scope: usize,
) -> Result<(), Error> {
        for i in &block.0 {
                match i {
                        BlockItem::D(declaration) => match declaration {
                                Declaration::V(variable_declaration) => {
                                        check_variable_declaration(variable_declaration, symbols, code, scope)?
                                }
                                Declaration::F(function_declaration) => {
                                        check_function_declaration(function_declaration, symbols, code, scope)?
                                }
                        },
                        BlockItem::S(astatement) => check_statement(astatement, symbols, code, scope)?,
                }
        }

        symbols.retain(|(_, item_scope), _| *item_scope < scope);

        Ok(())
}

fn check_statement<'b, 'a: 'b>(
        astatement: &AStatement,
        symbols: &mut HashMap<(&'b [u8], usize), Type>,
        code: &'a [u8],
        scope: usize,
) -> Result<(), Error> {
        match astatement {
                AStatement::Expr(aexpression) | AStatement::Return(aexpression) => {
                        check_expr(aexpression, symbols, code, scope)?
                }
                AStatement::I(IfStatement { condition, then, Else }) => {
                        check_expr(condition, symbols, code, scope)?;
                        check_statement(then, symbols, code, scope + 1)?;
                        if let Some(statement) = Else {
                                check_statement(statement, symbols, code, scope + 1)?;
                        }
                        symbols.retain(|(_, f), _| *f < scope + 1);
                }
                AStatement::Nul | AStatement::Break(_, _) | AStatement::Continue(_) => (),
                AStatement::Compound(ablock) => check_block(ablock, symbols, code, scope + 1)?,
                AStatement::While(aexpression, astatement, _) | AStatement::DoWhile(astatement, aexpression, _) => {
                        check_expr(aexpression, symbols, code, scope + 1)?;
                        check_statement(astatement, symbols, code, scope + 2)?;

                        symbols.retain(|(_, f), _| *f < scope + 1);
                }
                AStatement::F(For, _) => {
                        let For {
                                init,
                                condition,
                                post,
                                body,
                        } = &**For;

                        match init {
                                ForInit::D(variable_declaration) => {
                                        check_variable_declaration(variable_declaration, symbols, code, scope + 1)?
                                }
                                ForInit::E(Some(expr)) => check_expr(expr, symbols, code, scope + 2)?,
                                ForInit::E(None) => (),
                        }

                        if let Some(condition) = condition {
                                check_expr(condition, symbols, code, scope + 1)?;
                        }
                        if let Some(post) = post {
                                check_expr(post, symbols, code, scope + 1)?;
                        }

                        check_statement(body, symbols, code, scope + 2)?;

                        symbols.retain(|(_, f), _| *f < scope + 1);
                }
                AStatement::S(Switch {
                        value,
                        cases,
                        default,
                        label: _,
                }) => {
                        check_expr(value, symbols, code, scope)?;
                        for (_, i) in cases {
                                for j in i {
                                        check_statement(j, symbols, code, scope)?;
                                }
                        }
                        if let Some(statement) = default {
                                check_statement(statement, symbols, code, scope)?;
                        }
                }
        };
        Ok(())
}

fn name<'b, 'a: 'b>(code: &'a [u8], id: AIdentifier) -> &'b [u8] {
        let AIdentifier { start, len } = id;
        &code[start..start + len]
}

fn symbol_exists<'b, 'a: 'b>(
        code: &'a [u8],
        aidentifier: AIdentifier,
        symbols: &mut HashMap<(&'b [u8], usize), Type>,
        scope: usize,
) -> Result<Type, Error> {
        let AIdentifier { start, len } = aidentifier;
        let name = &code[start..start + len];

        for i in (0..=scope).rev() {
                if let Some(&found_type) = symbols.get(&(name, i)) {
                        return Ok(found_type);
                }
        }

        Err(Error::UndeclaredIdentifier(
                String::from_utf8(name.to_vec()).unwrap(),
                start,
        ))
}
