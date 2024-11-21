use std::collections::HashMap;

use crate::{
        parse::nodes::{
                ABlock, AExpression, AFactor, AIdentifier, AProgram, AStatement, BlockItem, Conditional, Declaration, ForInit, FunctionDeclaration, IfStatement, Switch, Unop,
                VariableDeclaration,
        },
        tactile::Identifier,
};

use super::Error;

pub fn resolve_identifiers<'b, 'a: 'b>(code: &'a [u8], program: &AProgram) -> Result<HashMap<(&'b [u8], usize), (Identifier, bool)>, Error> {
        let mut global_max_identifier = 0;
        let mut identifier_map = HashMap::new();

        for i in &program.functions {
                if let Some(body) = &i.body {
                        for j in &body.0 {
                                resolve_block_item(j, code, &mut identifier_map, &mut global_max_identifier)?;
                        }
                }
        }

        Ok(identifier_map)
}

fn resolve_block_item<'b, 'a: 'b>(
        block_item: &BlockItem,
        code: &'a [u8],
        identifier_map: &mut HashMap<(&'b [u8], usize), (Identifier, bool)>,
        global_max_identifier: &mut usize,
) -> Result<(), Error> {
        Ok(match block_item {
                BlockItem::D(declaration) => resolve_declaration(code, declaration, identifier_map, global_max_identifier, 0)?,
                BlockItem::S(astatement) => resolve_statement(code, astatement, identifier_map, global_max_identifier, 0)?,
        })
}

fn resolve_declaration<'b, 'a: 'b>(
        code: &'a [u8],
        declaration: &Declaration,
        identifier_map: &mut HashMap<(&'b [u8], usize), (Identifier, bool)>,
        global_max_identifier: &mut usize,
        scope: usize,
) -> Result<(), Error> {
        match declaration {
                Declaration::V(variable_declaration) => resolve_variable_declaration(code, variable_declaration, identifier_map, global_max_identifier, scope),
                Declaration::F(function_declaration) => resolve_function_declaration(code, function_declaration, identifier_map, global_max_identifier, scope),
        }
}

fn resolve_function_declaration<'b, 'a: 'b>(
        code: &'a [u8],
        declaration: &FunctionDeclaration,
        identifier_map: &mut HashMap<(&'b [u8], usize), (Identifier, bool)>,
        global_max_identifier: &mut usize,
        scope: usize,
) -> Result<(), Error> {
        let AIdentifier { start, len } = declaration.name;
        let name = &code[start..start + len];
        if identifier_exists(code, &declaration.name, identifier_map, scope).is_ok() {
                if let Some(curr_scope) = identifier_map.get(&(name, scope)) {
                        if !curr_scope.1 {
                                return Err(Error::DeclaredTwice(String::from_utf8(name.to_vec()).unwrap(), start + len));
                        }
                }
        };

        identifier_map.entry((name, scope)).insert_entry((
                {
                        let temp = Identifier(*global_max_identifier);
                        *global_max_identifier += 1;
                        temp
                },
                true,
        ));

        if let Some(thing) = &declaration.params {
                for i in thing {
                        let AIdentifier { start, len } = *i;
                        let name = &code[start..start + len];
                        if identifier_map.get(&(name, scope)).is_some() {
                                return Err(Error::DeclaredTwice(String::from_utf8(name.to_vec()).unwrap(), start));
                        }
                        let temp = *global_max_identifier;
                        *global_max_identifier += 1;

                        identifier_map.entry((name, scope)).insert_entry((Identifier(temp), false));
                }
        }

        if let Some(body) = &declaration.body {
                for i in &body.0 {
                        resolve_block_item(i, code, identifier_map, global_max_identifier)?
                }
        }

        Ok(())
}

fn resolve_variable_declaration<'b, 'a: 'b>(
        code: &'a [u8],
        declaration: &VariableDeclaration,
        identifier_map: &mut HashMap<(&'b [u8], usize), (Identifier, bool)>,
        global_max_identifier: &mut usize,
        scope: usize,
) -> Result<(), Error> {
        let AIdentifier { start, len } = declaration.id;
        let name = &code[start..start + len];
        if identifier_map.get(&(name, scope)).is_some() {
                return Err(Error::DeclaredTwice(String::from_utf8(name.to_vec()).unwrap(), start));
        }
        let temp = *global_max_identifier;
        *global_max_identifier += 1;

        identifier_map.entry((name, scope)).insert_entry((Identifier(temp), false));

        if let Some(extract) = &declaration.init {
                () = resolve_exp(code, extract, identifier_map, scope)?;
        }

        Ok(())
}

fn resolve_statement<'b, 'a: 'b>(
        code: &'a [u8],
        statement: &AStatement,
        identifier_map: &mut HashMap<(&'b [u8], usize), (Identifier, bool)>,
        max_identifier: &mut usize,
        scope: usize,
) -> Result<(), Error> {
        match statement {
                AStatement::Return(expr) | AStatement::Expr(expr) => resolve_exp(code, expr, identifier_map, scope),
                AStatement::I(if_statement) => {
                        let IfStatement { condition, then, Else } = if_statement;
                        resolve_exp(code, condition, identifier_map, scope)?;
                        resolve_statement(code, then, identifier_map, max_identifier, scope)?;
                        if let Some(else_statement) = Else {
                                resolve_statement(code, else_statement, identifier_map, max_identifier, scope)?;
                        }

                        Ok(())
                }
                AStatement::Nul | AStatement::Break(..) | AStatement::Continue(_) => Ok(()),
                AStatement::Compound(ABlock(block)) => {
                        let inner_scope = scope + 1;

                        for i in block {
                                match i {
                                        BlockItem::D(declaration) => resolve_declaration(code, declaration, identifier_map, max_identifier, inner_scope)?,
                                        BlockItem::S(astatement) => resolve_statement(code, astatement, identifier_map, max_identifier, inner_scope)?,
                                }
                        }

                        identifier_map.retain(|(_, scope), _| scope != &inner_scope);

                        Ok(())
                }
                AStatement::While(aexpression, astatement, _) => {
                        () = resolve_exp(code, aexpression, identifier_map, scope + 1)?;
                        () = resolve_statement(code, astatement, identifier_map, max_identifier, scope + 2)?;
                        identifier_map.retain(|(_, scope), _| scope != &(scope + 2));
                        identifier_map.retain(|(_, scope), _| scope != &(scope + 1));

                        Ok(())
                }
                AStatement::DoWhile(astatement, aexpression, _) => {
                        () = resolve_exp(code, aexpression, identifier_map, scope + 1)?;
                        () = resolve_statement(code, astatement, identifier_map, max_identifier, scope + 2)?;
                        identifier_map.retain(|(_, scope), _| scope != &(scope + 2));
                        identifier_map.retain(|(_, scope), _| scope != &(scope + 1));

                        Ok(())
                }
                AStatement::F(boxed_for, _) => {
                        let header_scope = scope + 1;

                        match &boxed_for.init {
                                ForInit::D(declaration) => resolve_variable_declaration(code, declaration, identifier_map, max_identifier, header_scope)?,
                                ForInit::E(Some(aexpression)) => resolve_exp(code, aexpression, identifier_map, header_scope)?,
                                ForInit::E(None) => {}
                        }

                        if let Some(cond) = &boxed_for.condition {
                                let () = resolve_exp(code, cond, identifier_map, header_scope)?;
                        }
                        if let Some(post) = &boxed_for.post {
                                let () = resolve_exp(code, post, identifier_map, header_scope)?;
                        }

                        let body_scope = header_scope + 1;
                        () = resolve_statement(code, &boxed_for.body, identifier_map, max_identifier, body_scope)?;

                        identifier_map.retain(|(_, scope), _| scope != &header_scope);
                        identifier_map.retain(|(_, scope), _| scope != &body_scope);

                        Ok(())
                }
                AStatement::S(switch) => {
                        let Switch { value, cases, default, .. } = switch;
                        resolve_exp(code, value, identifier_map, scope)?;
                        for (_, statements) in cases {
                                for j in statements {
                                        resolve_statement(code, j, identifier_map, max_identifier, scope)?;
                                }
                        }
                        if let Some(default) = default {
                                resolve_statement(code, default, identifier_map, max_identifier, scope)?;
                        }
                        Ok(())
                }
        }
}

fn resolve_exp(code: &[u8], expr: &AExpression, identifier_map: &mut HashMap<(&[u8], usize), (Identifier, bool)>, scope: usize) -> Result<(), Error> {
        match expr {
                AExpression::F(afactor) => match afactor {
                        AFactor::Expr(aexpression) => resolve_exp(code, aexpression, identifier_map, scope),
                        AFactor::Id(aidentifier) => identifier_exists(code, aidentifier, identifier_map, scope).map(|_| ()),
                        AFactor::Unop(unop, afactor) => is_valid_lvalue_unop(code, *unop, *afactor.clone(), identifier_map, scope),
                        AFactor::Constant(_) => Ok(()),
                },
                AExpression::Assignment(left, right) => {
                        is_valid_lvalue_assignment(code, left, identifier_map, scope)?;
                        resolve_exp(code, right, identifier_map, scope)
                }
                AExpression::BinOp(_, left, right) => {
                        resolve_exp(code, left, identifier_map, scope)?;
                        resolve_exp(code, right, identifier_map, scope)
                }
                AExpression::C(Conditional { condition, True, False }) => {
                        resolve_exp(code, condition, identifier_map, scope)?;
                        resolve_exp(code, True, identifier_map, scope)?;
                        resolve_exp(code, False, identifier_map, scope)
                }
                AExpression::OpAssignment(_binop, left, right) => {
                        is_valid_lvalue_assignment(code, left, identifier_map, scope)?;
                        resolve_exp(code, right, identifier_map, scope)
                }
                AExpression::FunctionCall(aidentifier, vec) => {
                        let AIdentifier { start, len } = *aidentifier;
                        let name = &code[start..start + len];

                        if identifier_map.get(&(name, scope)).is_none() {
                                return Err(Error::UndeclaredIdentifier(String::from_utf8(name.to_vec()).unwrap(), start + len));
                        }

                        if let Some(cases) = vec {
                                for i in cases {
                                        resolve_exp(code, i, identifier_map, scope)?;
                                }
                        }

                        Ok(())
                }
        }
}

fn is_valid_lvalue_assignment(code: &[u8], left: &AExpression, identifier_map: &mut HashMap<(&[u8], usize), (Identifier, bool)>, scope: usize) -> Result<(), Error> {
        match left {
                AExpression::F(afactor) => match afactor {
                        AFactor::Expr(expr) => is_valid_lvalue_assignment(code, expr, identifier_map, scope),
                        AFactor::Id(aidentifier) => identifier_exists(code, aidentifier, identifier_map, scope).map(|_| ()),
                        AFactor::Constant(..) | AFactor::Unop(..) => Err(Error::InvalidLValueExpr(left.clone())),
                },
                AExpression::Assignment(left, right) => {
                        resolve_exp(code, left, identifier_map, scope)?;
                        resolve_exp(code, right, identifier_map, scope)
                }
                AExpression::C(_) | AExpression::BinOp(..) | AExpression::OpAssignment(..) => Err(Error::InvalidLValueExpr(left.clone())),
                AExpression::FunctionCall(aidentifier, vec) => todo!(),
        }
}

fn is_valid_lvalue_unop(code: &[u8], unop: Unop, factor: AFactor, identifier_map: &mut HashMap<(&[u8], usize), (Identifier, bool)>, scope: usize) -> Result<(), Error> {
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
                        is_valid_lvalue_unop(code, innerunop, *afactor, identifier_map, scope)?;

                        Ok(())
                }
                AFactor::Expr(aexpression) => {
                        resolve_exp(code, &aexpression, identifier_map, scope)?;
                        match *aexpression {
                                AExpression::F(afactor) => is_valid_lvalue_unop(code, unop, afactor, identifier_map, scope),
                                AExpression::Assignment(..) | AExpression::C(_) | AExpression::OpAssignment(..) => Err(Error::InvalidLValueExpr(*aexpression)),
                                AExpression::BinOp(_binop, left, right) => {
                                        match unop {
                                                Unop::Negate | Unop::Complement | Unop::Not => {}
                                                Unop::IncrementPre | Unop::IncrementPost | Unop::DecrementPre | Unop::DecrementPost => return Err(Error::InvalidLValueFactor(factor)),
                                        }
                                        resolve_exp(code, &left, identifier_map, scope)?;
                                        resolve_exp(code, &right, identifier_map, scope)
                                }
                                AExpression::FunctionCall(aidentifier, vec) => todo!(),
                        }
                }
                AFactor::Id(aidentifier) => identifier_exists(code, &aidentifier, identifier_map, scope).map(|_| ()),
        }
}

fn identifier_exists(code: &[u8], aidentifier: &AIdentifier, identifier_map: &mut HashMap<(&[u8], usize), (Identifier, bool)>, scope: usize) -> Result<(Identifier, bool), Error> {
        let &AIdentifier { start, len } = aidentifier;
        let name = &code[start..start + len];

        for i in 0..=scope {
                if let Some(thing) = identifier_map.get(&(name, i)) {
                        return Ok(*thing);
                }
        }

        Err(Error::UndeclaredIdentifier(String::from_utf8(name.to_vec()).unwrap(), start))
}
