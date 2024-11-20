use std::collections::HashMap;

use crate::{
        parse::nodes::{ABlock, AExpression, AFactor, AIdentifier, AProgram, AStatement, BlockItem, Conditional, Declaration, ForInit, IfStatement, Switch, Unop, VariableDeclaration},
        tactile::Identifier,
};

use super::Error;

pub fn resolve_variables<'b, 'a: 'b>(code: &'a [u8], program: &AProgram) -> Result<HashMap<(&'b [u8], usize), Identifier>, Error> {
        let mut global_max_identifier = 0;
        let mut variable_map = HashMap::new();

        for i in &program.functions {
                if let Some(body) = &i.body {
                        for j in &body.0 {
                                match j {
                                        BlockItem::D(declaration) => resolve_declaration(code, declaration, &mut variable_map, &mut global_max_identifier, 0)?,
                                        BlockItem::S(astatement) => resolve_statement(code, astatement, &mut variable_map, &mut global_max_identifier, 0)?,
                                }
                        }
                }
        }

        Ok(variable_map)
}

fn resolve_declaration<'b, 'a: 'b>(
        code: &'a [u8],
        declaration: &Declaration,
        variable_map: &mut HashMap<(&'b [u8], usize), Identifier>,
        global_max_identifier: &mut usize,
        scope: usize,
) -> Result<(), Error> {
        todo!()
}

fn resolve_variable_declaration<'b, 'a: 'b>(
        code: &'a [u8],
        declaration: &VariableDeclaration,
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

fn resolve_statement<'b, 'a: 'b>(
        code: &'a [u8],
        statement: &AStatement,
        variable_map: &mut HashMap<(&'b [u8], usize), Identifier>,
        max_identifier: &mut usize,
        scope: usize,
) -> Result<(), Error> {
        match statement {
                AStatement::Return(expr) | AStatement::Expr(expr) => resolve_exp(code, expr, variable_map, scope),
                AStatement::I(if_statement) => {
                        let IfStatement { condition, then, Else } = if_statement;
                        resolve_exp(code, condition, variable_map, scope)?;
                        resolve_statement(code, then, variable_map, max_identifier, scope)?;
                        if let Some(else_statement) = Else {
                                resolve_statement(code, else_statement, variable_map, max_identifier, scope)?;
                        }

                        Ok(())
                }
                AStatement::Nul | AStatement::Break(..) | AStatement::Continue(_) => Ok(()),
                AStatement::Compound(ABlock(block)) => {
                        let inner_scope = scope + 1;

                        for i in block {
                                match i {
                                        BlockItem::D(declaration) => resolve_declaration(code, declaration, variable_map, max_identifier, inner_scope)?,
                                        BlockItem::S(astatement) => resolve_statement(code, astatement, variable_map, max_identifier, inner_scope)?,
                                }
                        }

                        variable_map.retain(|(_, scope), _| scope != &inner_scope);

                        Ok(())
                }
                AStatement::While(aexpression, astatement, _) => {
                        () = resolve_exp(code, aexpression, variable_map, scope + 1)?;
                        () = resolve_statement(code, astatement, variable_map, max_identifier, scope + 2)?;
                        variable_map.retain(|(_, scope), _| scope != &(scope + 2));
                        variable_map.retain(|(_, scope), _| scope != &(scope + 1));

                        Ok(())
                }
                AStatement::DoWhile(astatement, aexpression, _) => {
                        () = resolve_exp(code, aexpression, variable_map, scope + 1)?;
                        () = resolve_statement(code, astatement, variable_map, max_identifier, scope + 2)?;
                        variable_map.retain(|(_, scope), _| scope != &(scope + 2));
                        variable_map.retain(|(_, scope), _| scope != &(scope + 1));

                        Ok(())
                }
                AStatement::F(boxed_for, _) => {
                        let header_scope = scope + 1;

                        match &boxed_for.init {
                                ForInit::D(declaration) => resolve_variable_declaration(code, declaration, variable_map, max_identifier, header_scope)?,
                                ForInit::E(Some(aexpression)) => resolve_exp(code, aexpression, variable_map, header_scope)?,
                                ForInit::E(None) => {}
                        }

                        if let Some(cond) = &boxed_for.condition {
                                let () = resolve_exp(code, cond, variable_map, header_scope)?;
                        }
                        if let Some(post) = &boxed_for.post {
                                let () = resolve_exp(code, post, variable_map, header_scope)?;
                        }

                        let body_scope = header_scope + 1;
                        () = resolve_statement(code, &boxed_for.body, variable_map, max_identifier, body_scope)?;

                        variable_map.retain(|(_, scope), _| scope != &header_scope);
                        variable_map.retain(|(_, scope), _| scope != &body_scope);

                        Ok(())
                }
                AStatement::S(switch) => {
                        let Switch { value, cases, default, .. } = switch;
                        resolve_exp(code, value, variable_map, scope)?;
                        for (_, statements) in cases {
                                for j in statements {
                                        resolve_statement(code, j, variable_map, max_identifier, scope)?;
                                }
                        }
                        if let Some(default) = default {
                                resolve_statement(code, default, variable_map, max_identifier, scope)?;
                        }
                        Ok(())
                }
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
                AExpression::FunctionCall(aidentifier, vec) => todo!(),
        }
}

fn is_valid_lvalue_assignment(code: &[u8], left: &AExpression, variable_map: &mut HashMap<(&[u8], usize), Identifier>, scope: usize) -> Result<(), Error> {
        match left {
                AExpression::F(afactor) => match afactor {
                        AFactor::Expr(expr) => is_valid_lvalue_assignment(code, expr, variable_map, scope),
                        AFactor::Id(aidentifier) => variable_exists(code, aidentifier, variable_map, scope),
                        AFactor::Constant(..) | AFactor::Unop(..) => Err(Error::InvalidLValueExpr(left.clone())),
                },
                AExpression::Assignment(left, right) => {
                        resolve_exp(code, left, variable_map, scope)?;
                        resolve_exp(code, right, variable_map, scope)
                }
                AExpression::C(_) | AExpression::BinOp(..) | AExpression::OpAssignment(..) => Err(Error::InvalidLValueExpr(left.clone())),
                AExpression::FunctionCall(aidentifier, vec) => todo!(),
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
                                                Unop::IncrementPre | Unop::IncrementPost | Unop::DecrementPre | Unop::DecrementPost => return Err(Error::InvalidLValueFactor(factor)),
                                        }
                                        resolve_exp(code, &left, variable_map, scope)?;
                                        resolve_exp(code, &right, variable_map, scope)
                                }
                                AExpression::FunctionCall(aidentifier, vec) => todo!(),
                        }
                }
                AFactor::Id(aidentifier) => variable_exists(code, &aidentifier, variable_map, scope),
        }
}

fn variable_exists(code: &[u8], aidentifier: &AIdentifier, variable_map: &mut HashMap<(&[u8], usize), Identifier>, scope: usize) -> Result<(), Error> {
        let &AIdentifier { start, len } = aidentifier;
        let name = &code[start..start + len];

        for i in 0..=scope {
                if variable_map.get(&(name, i)).is_some() {
                        return Ok(());
                }
        }

        Err(Error::UndeclaredVariable(String::from_utf8(name.to_vec()).unwrap(), start))
}
