use std::{collections::HashMap, hash::BuildHasher};

use crate::{
        parse::nodes::{
                ABlock, AConstant, AExpression, AFactor, AIdentifier, AProgram, AStatement, Binop, BlockItem, Conditional, IfStatement,
                Unop,
        },
        semantic_analysis::SemanticallyAnalyzed,
        Program, State,
};

#[derive(Debug, Clone, Copy)]
pub struct Identifier(pub usize);

#[derive(Debug)]
pub struct TACTILE {
        pub code: Vec<u8>,
        pub program: TACTILEProgram,
}
impl State for TACTILE {}

#[derive(Debug, Clone, Copy)]
pub enum Value {
        Constant(Constant),
        Var(Identifier),
}

#[derive(Debug, Clone, Copy)]
pub enum Constant {
        A(AConstant),
        S(i64),
}

#[derive(Debug, Clone, Copy)]
pub enum TACTILEInstruction {
        Return(Value),
        Unary(Unop, Value, Value),
        Binary(Binop, Value, Value, Value),
        Copy(Value, Value),
        Jump(Label),
        JumpIfZero(Value, Label),
        JumpIfNotZero(Value, Label),
        Label(Label),
}

#[derive(Debug, Clone, Copy)]
pub struct Label(pub usize);

#[derive(Debug, Clone, Copy)]
struct TACTILELoopLabel(pub usize, pub usize, pub usize);

#[derive(Debug, Clone)]
pub struct TACTILEFunction {
        pub identifier: AIdentifier,
        pub instructions: Vec<TACTILEInstruction>,
}

#[derive(Debug, Clone)]
pub struct TACTILEProgram {
        pub function: TACTILEFunction,
}

fn emit_tactile_expr<'b, 'a: 'b, S: BuildHasher>(
        code: &'a [u8],
        value: AExpression,
        instructions: &mut Vec<TACTILEInstruction>,
        max_identifier: &mut usize,
        max_label: &mut usize,
        named_variable_map: &mut HashMap<(&'b [u8], usize), Identifier, S>,
        scope: usize,
) -> Value {
        match value {
                AExpression::F(AFactor::Constant(n)) => Value::Constant(Constant::A(n)),
                AExpression::F(AFactor::Unop(unop, afactor)) => {
                        let src = emit_tactile_expr(
                                code,
                                AExpression::F(*afactor),
                                instructions,
                                max_identifier,
                                max_label,
                                named_variable_map,
                                scope,
                        );
                        let dst = Value::Var(Identifier(*max_identifier));
                        *max_identifier += 1;
                        instructions.push(TACTILEInstruction::Unary(unop, src, dst));
                        dst
                }
                AExpression::BinOp(binop, src, dst) => match binop {
                        Binop::LogicalOr => {
                                let false_label = Label(*max_label);
                                *max_label += 1;
                                let end_label = Label(*max_label);
                                *max_label += 1;

                                let v1 = emit_tactile_expr(code, *src, instructions, max_identifier, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfNotZero(v1, false_label));
                                let v2 = emit_tactile_expr(code, *dst, instructions, max_identifier, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfNotZero(v2, false_label));

                                let dst = Value::Var(Identifier(*max_identifier));
                                *max_identifier += 1;

                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(1)), dst));
                                instructions.push(TACTILEInstruction::Jump(end_label));
                                instructions.push(TACTILEInstruction::Label(false_label));
                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(0)), dst));
                                instructions.push(TACTILEInstruction::Label(end_label));

                                dst
                        }
                        Binop::LogicalAnd => {
                                let false_label = Label(*max_label);
                                *max_label += 1;
                                let end_label = Label(*max_label);
                                *max_label += 1;

                                let v1 = emit_tactile_expr(code, *src, instructions, max_identifier, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfZero(v1, false_label));
                                let v2 = emit_tactile_expr(code, *dst, instructions, max_identifier, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfZero(v2, false_label));

                                let dst = Value::Var(Identifier(*max_identifier));
                                *max_identifier += 1;

                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(1)), dst));
                                instructions.push(TACTILEInstruction::Jump(end_label));
                                instructions.push(TACTILEInstruction::Label(false_label));
                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(0)), dst));
                                instructions.push(TACTILEInstruction::Label(end_label));

                                dst
                        }
                        _ => {
                                let v1 = emit_tactile_expr(code, *src, instructions, max_identifier, max_label, named_variable_map, scope);
                                let v2 = emit_tactile_expr(code, *dst, instructions, max_identifier, max_label, named_variable_map, scope);
                                let dst = Value::Var(Identifier(*max_identifier));
                                *max_identifier += 1;
                                instructions.push(TACTILEInstruction::Binary(binop, v1, v2, dst));
                                dst
                        }
                },
                AExpression::F(AFactor::Expr(expr)) => {
                        emit_tactile_expr(code, *expr, instructions, max_identifier, max_label, named_variable_map, scope)
                }
                AExpression::Assignment(lval, rval) => {
                        let left = emit_tactile_expr(code, *lval, instructions, max_identifier, max_label, named_variable_map, scope);
                        let right = emit_tactile_expr(code, *rval, instructions, max_identifier, max_label, named_variable_map, scope);
                        instructions.push(TACTILEInstruction::Copy(right, left));
                        left
                }
                AExpression::F(AFactor::Id(id)) => {
                        let AIdentifier { start, len } = id;
                        let name = &code[start..start + len];
                        let entry = (name, scope);

                        let entered = named_variable_map.entry(entry).or_insert({
                                let id = Identifier(*max_identifier);
                                *max_identifier += 1;
                                id
                        });

                        Value::Var(*entered)
                }
                AExpression::OpAssignment(binop, left, right) => {
                        let left = emit_tactile_expr(code, *left, instructions, max_identifier, max_label, named_variable_map, scope);
                        let right = emit_tactile_expr(code, *right, instructions, max_identifier, max_label, named_variable_map, scope);

                        instructions.push(TACTILEInstruction::Binary(binop, left, right, left));
                        left
                }
                AExpression::C(Conditional { condition, True, False }) => {
                        let end_label = Label(*max_label);
                        *max_label += 1;
                        let else_label = Label(*max_label);
                        *max_label += 1;

                        let result = Value::Var(Identifier(*max_identifier));
                        *max_identifier += 1;

                        let c = emit_tactile_expr(code, *condition, instructions, max_identifier, max_label, named_variable_map, scope);
                        instructions.push(TACTILEInstruction::JumpIfZero(c, else_label));

                        let val1 = emit_tactile_expr(code, *True, instructions, max_identifier, max_label, named_variable_map, scope);
                        instructions.extend_from_slice(&[
                                TACTILEInstruction::Copy(val1, result),
                                TACTILEInstruction::Jump(end_label),
                                TACTILEInstruction::Label(else_label),
                        ]);

                        let val2 = emit_tactile_expr(code, *False, instructions, max_identifier, max_label, named_variable_map, scope);
                        instructions.extend_from_slice(&[TACTILEInstruction::Copy(val2, result), TACTILEInstruction::Label(end_label)]);

                        result
                }
        }
}

fn tactile_program(program: AProgram, code: &[u8], max_label: &mut usize) -> TACTILEProgram {
        let value = program.function;
        let mut instructions = vec![];
        let mut global_max_identifier = 1;
        let mut named_variable_map = HashMap::new();
        let mut loop_labels = vec![];
        let scope = 0;

        for i in value.function_body.0 {
                match i {
                        BlockItem::D(declaration) => {
                                if let Some(init) = declaration.init {
                                        let var = emit_tactile_expr(
                                                code,
                                                AExpression::F(AFactor::Id(declaration.id)),
                                                &mut instructions,
                                                &mut global_max_identifier,
                                                max_label,
                                                &mut named_variable_map,
                                                scope,
                                        );
                                        let expr = emit_tactile_expr(
                                                code,
                                                init,
                                                &mut instructions,
                                                &mut global_max_identifier,
                                                max_label,
                                                &mut named_variable_map,
                                                scope,
                                        );
                                        instructions.push(TACTILEInstruction::Copy(expr, var));
                                }
                        }
                        BlockItem::S(astatement) => emit_tactile_statement(
                                code,
                                astatement,
                                &mut instructions,
                                &mut global_max_identifier,
                                max_label,
                                &mut named_variable_map,
                                &mut loop_labels,
                                scope,
                        ),
                }
        }

        instructions.push(TACTILEInstruction::Return(Value::Constant(Constant::S(0))));

        TACTILEProgram {
                function: TACTILEFunction {
                        identifier: value.identifier,
                        instructions,
                },
        }
}

fn emit_tactile_statement<'b, 'a: 'b>(
        code: &'a [u8],
        value: AStatement,
        instructions: &mut Vec<TACTILEInstruction>,
        max_identifier: &mut usize,
        max_label: &mut usize,
        named_variable_map: &mut HashMap<(&'b [u8], usize), Identifier>,
        loop_labels: &mut Vec<TACTILELoopLabel>,
        scope: usize,
) {
        match value {
                AStatement::Return(aexpression) => {
                        let val = emit_tactile_expr(
                                code,
                                aexpression,
                                instructions,
                                max_identifier,
                                max_label,
                                named_variable_map,
                                scope,
                        );
                        instructions.push(TACTILEInstruction::Return(val));
                }
                AStatement::Expr(aexpression) => {
                        let _ = emit_tactile_expr(
                                code,
                                aexpression,
                                instructions,
                                max_identifier,
                                max_label,
                                named_variable_map,
                                scope,
                        );
                }
                AStatement::Nul => {}
                AStatement::I(IfStatement { condition, then, Else }) => {
                        if Else.is_none() {
                                let end = Label(*max_label);
                                *max_label += 1;

                                let c = emit_tactile_expr(
                                        code,
                                        condition,
                                        instructions,
                                        max_identifier,
                                        max_label,
                                        named_variable_map,
                                        scope,
                                );
                                instructions.push(TACTILEInstruction::JumpIfZero(c, end));

                                emit_tactile_statement(
                                        code,
                                        *then,
                                        instructions,
                                        max_identifier,
                                        max_label,
                                        named_variable_map,
                                        loop_labels,
                                        scope,
                                );

                                instructions.push(TACTILEInstruction::Label(end));

                                return;
                        }

                        let else_statement = Else.unwrap();
                        let end_label = Label(*max_label);
                        *max_label += 1;
                        let else_label = Label(*max_label);
                        *max_label += 1;

                        let c = emit_tactile_expr(code, condition, instructions, max_identifier, max_label, named_variable_map, scope);
                        instructions.push(TACTILEInstruction::JumpIfZero(c, else_label));

                        emit_tactile_statement(
                                code,
                                *then,
                                instructions,
                                max_identifier,
                                max_label,
                                named_variable_map,
                                loop_labels,
                                scope,
                        );
                        instructions.extend_from_slice(&[TACTILEInstruction::Jump(end_label), TACTILEInstruction::Label(else_label)]);

                        emit_tactile_statement(
                                code,
                                *else_statement,
                                instructions,
                                max_identifier,
                                max_label,
                                named_variable_map,
                                loop_labels,
                                scope,
                        );
                        instructions.push(TACTILEInstruction::Label(end_label));
                }
                AStatement::Compound(ABlock(block)) => {
                        let inner_scope = scope + 1;
                        for i in block {
                                match i {
                                        BlockItem::D(declaration) => {
                                                if let Some(init) = declaration.init {
                                                        let var = emit_tactile_expr(
                                                                code,
                                                                AExpression::F(AFactor::Id(declaration.id)),
                                                                instructions,
                                                                max_identifier,
                                                                max_label,
                                                                named_variable_map,
                                                                inner_scope,
                                                        );
                                                        let expr = emit_tactile_expr(
                                                                code,
                                                                init,
                                                                instructions,
                                                                max_identifier,
                                                                max_label,
                                                                named_variable_map,
                                                                inner_scope,
                                                        );
                                                        instructions.push(TACTILEInstruction::Copy(expr, var));
                                                }
                                        }
                                        BlockItem::S(astatement) => emit_tactile_statement(
                                                code,
                                                astatement,
                                                instructions,
                                                max_identifier,
                                                max_label,
                                                named_variable_map,
                                                loop_labels,
                                                inner_scope,
                                        ),
                                }
                        }
                        named_variable_map.retain(|(_, scope), _| scope != &inner_scope);
                }
                AStatement::While(aexpression, astatement, loop_label) => todo!(),
                AStatement::F(_, loop_label) => todo!(),
                AStatement::Break(loop_label) | AStatement::Continue(loop_label) => {
                        let mut label = None;
                        for i in loop_labels.iter().rev() {
                                if i.0 == loop_label.0 {
                                        label = Some(i.0);
                                        break;
                                }
                        }
                        let label = label.expect("Logic bug. We should have found all breaks and continues outside loops before");
                        instructions.push(TACTILEInstruction::Jump(Label(label)));
                }
                AStatement::DoWhile(astatement, aexpression, loop_label) => todo!(),
        }
}

pub fn TACTILE(program: Program<SemanticallyAnalyzed>, mut max_label: usize) -> Program<TACTILE> {
        let code = program.state.code;
        Program {
                operation: program.operation,
                state: TACTILE {
                        program: tactile_program(program.state.program, &code[..], &mut max_label),
                        code,
                },
        }
}
