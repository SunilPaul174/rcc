use std::{collections::HashMap, hash::BuildHasher};

use crate::{
        parse::nodes::{
                ABlock, AConstant, AExpression, AFactor, AIdentifier, AProgram, AStatement, Binop, BlockItem, Conditional, Declaration, For, ForInit, IfStatement, LoopLabel, Unop,
        },
        semantic_analysis::SemanticallyAnalyzed,
        Program, State,
};

#[derive(Debug, Clone, Copy)]
pub struct Identifier(pub usize);

#[derive(Debug, Clone)]
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
        L(Label),
}

#[derive(Debug, Clone, Copy)]
pub struct Label(pub usize);

#[derive(Debug, Clone, Copy)]
struct TACTILELoopLabel {
        pub begin: usize,
        pub break_label: usize,
        pub continue_label: usize,
}

fn tactilify_loop_label(label: LoopLabel, max_label: &mut usize) -> TACTILELoopLabel {
        let break_label = *max_label;
        let continue_label = *max_label + 1;
        let begin = label.0;
        *max_label += 2;
        TACTILELoopLabel { begin, break_label, continue_label }
}

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
        max_id: &mut usize,
        max_label: &mut usize,
        named_variable_map: &mut HashMap<(&'b [u8], usize), Identifier, S>,
        scope: usize,
) -> Value {
        match value {
                AExpression::F(AFactor::Constant(n)) => Value::Constant(Constant::A(n)),
                AExpression::F(AFactor::Unop(unop, afactor)) => match unop {
                        Unop::Negate | Unop::Complement | Unop::Not => {
                                let src = emit_tactile_expr(&code, AExpression::F(*afactor), instructions, max_id, max_label, named_variable_map, scope);
                                let dst = Value::Var(new_id(max_id));
                                instructions.push(TACTILEInstruction::Unary(unop, src, dst));
                                dst
                        }
                        Unop::IncrementPre | Unop::DecrementPre => {
                                let left = emit_tactile_expr(code, AExpression::F(*afactor), instructions, max_id, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::Unary(unop, left, left));
                                left
                        }
                        Unop::IncrementPost | Unop::DecrementPost => {
                                let src = emit_tactile_expr(&code, AExpression::F(*afactor), instructions, max_id, max_label, named_variable_map, scope);
                                let dst = Value::Var(new_id(max_id));
                                instructions.push(TACTILEInstruction::Copy(src, dst));
                                instructions.push(TACTILEInstruction::Unary(unop, src, src));
                                dst
                        }
                },
                AExpression::BinOp(binop, left, right) => match binop {
                        Binop::LogicalOr => {
                                let false_label = new_label(max_label);
                                let end_label = new_label(max_label);

                                let v1 = emit_tactile_expr(code, *left, instructions, max_id, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfNotZero(v1, false_label));
                                let v2 = emit_tactile_expr(code, *right, instructions, max_id, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfNotZero(v2, false_label));

                                let dst = Value::Var(new_id(max_id));

                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(1)), dst));
                                instructions.push(TACTILEInstruction::Jump(end_label));
                                instructions.push(TACTILEInstruction::L(false_label));
                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(0)), dst));
                                instructions.push(TACTILEInstruction::L(end_label));

                                dst
                        }
                        Binop::LogicalAnd => {
                                let false_label = new_label(max_label);
                                let end_label = new_label(max_label);

                                let v1 = emit_tactile_expr(code, *left, instructions, max_id, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfZero(v1, false_label));
                                let v2 = emit_tactile_expr(code, *right, instructions, max_id, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfZero(v2, false_label));

                                let dst = Value::Var(new_id(max_id));

                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(1)), dst));
                                instructions.push(TACTILEInstruction::Jump(end_label));
                                instructions.push(TACTILEInstruction::L(false_label));
                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(0)), dst));
                                instructions.push(TACTILEInstruction::L(end_label));

                                dst
                        }
                        _ => {
                                let v1 = emit_tactile_expr(code, *left, instructions, max_id, max_label, named_variable_map, scope);
                                let v2 = emit_tactile_expr(code, *right, instructions, max_id, max_label, named_variable_map, scope);
                                let dst = Value::Var(new_id(max_id));
                                instructions.push(TACTILEInstruction::Binary(binop, v1, v2, dst));
                                dst
                        }
                },
                AExpression::F(AFactor::Expr(expr)) => emit_tactile_expr(code, *expr, instructions, max_id, max_label, named_variable_map, scope),
                AExpression::Assignment(lval, rval) => {
                        let left = emit_tactile_expr(code, *lval, instructions, max_id, max_label, named_variable_map, scope);
                        let right = emit_tactile_expr(code, *rval, instructions, max_id, max_label, named_variable_map, scope);
                        instructions.push(TACTILEInstruction::Copy(right, left));
                        left
                }
                AExpression::F(AFactor::Id(id)) => {
                        let AIdentifier { start, len } = id;
                        let name = &code[start..start + len];
                        let mut potential = named_variable_map.keys().filter(|(f, _)| *f == name).peekable();

                        if potential.peek().is_none() {
                                let entered = named_variable_map.entry((name, scope)).insert_entry(new_id(max_id));
                                return Value::Var(*entered.get());
                        }

                        let max = potential.max_by_key(|f| f.1).unwrap();

                        Value::Var(*named_variable_map.get(max).unwrap())
                }
                AExpression::OpAssignment(binop, left, right) => {
                        let left = emit_tactile_expr(code, *left, instructions, max_id, max_label, named_variable_map, scope);
                        let right = emit_tactile_expr(code, *right, instructions, max_id, max_label, named_variable_map, scope);

                        instructions.push(TACTILEInstruction::Binary(binop, left, right, left));
                        left
                }
                AExpression::C(Conditional { condition, True, False }) => {
                        let end_label = new_label(max_label);
                        let else_label = new_label(max_label);

                        let result = Value::Var(new_id(max_id));

                        let c = emit_tactile_expr(code, *condition, instructions, max_id, max_label, named_variable_map, scope);
                        instructions.push(TACTILEInstruction::JumpIfZero(c, else_label));

                        let val1 = emit_tactile_expr(code, *True, instructions, max_id, max_label, named_variable_map, scope);
                        instructions.extend_from_slice(&[TACTILEInstruction::Copy(val1, result), TACTILEInstruction::Jump(end_label), TACTILEInstruction::L(else_label)]);

                        let val2 = emit_tactile_expr(code, *False, instructions, max_id, max_label, named_variable_map, scope);
                        instructions.extend_from_slice(&[TACTILEInstruction::Copy(val2, result), TACTILEInstruction::L(end_label)]);

                        result
                }
        }
}

fn tactile_program<'b, 'a: 'b>(program: AProgram, code: &'a [u8], max_label: &mut usize, mut named_variable_map: HashMap<(&'b [u8], usize), Identifier>) -> TACTILEProgram {
        let value = program.function;
        let mut instructions = vec![];
        let mut global_max_identifier = 1;
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
                                        let expr = emit_tactile_expr(code, init, &mut instructions, &mut global_max_identifier, max_label, &mut named_variable_map, scope);
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
                        let val = emit_tactile_expr(code, aexpression, instructions, max_identifier, max_label, named_variable_map, scope);
                        instructions.push(TACTILEInstruction::Return(val));
                }
                AStatement::Expr(aexpression) => {
                        let _ = emit_tactile_expr(code, aexpression, instructions, max_identifier, max_label, named_variable_map, scope);
                }
                AStatement::Nul => {}
                AStatement::I(IfStatement { condition, then, Else }) => {
                        if Else.is_none() {
                                let end = new_label(max_label);

                                let c = emit_tactile_expr(code, condition, instructions, max_identifier, max_label, named_variable_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfZero(c, end));

                                emit_tactile_statement(code, *then, instructions, max_identifier, max_label, named_variable_map, loop_labels, scope + 1);

                                named_variable_map.retain(|&(_, b), _| b <= scope);

                                instructions.push(TACTILEInstruction::L(end));

                                return;
                        }

                        let else_statement = Else.unwrap();
                        let end_label = new_label(max_label);
                        let else_label = new_label(max_label);

                        let c = emit_tactile_expr(code, condition, instructions, max_identifier, max_label, named_variable_map, scope);
                        instructions.push(TACTILEInstruction::JumpIfZero(c, else_label));

                        emit_tactile_statement(code, *then, instructions, max_identifier, max_label, named_variable_map, loop_labels, scope + 1);
                        instructions.extend_from_slice(&[TACTILEInstruction::Jump(end_label), TACTILEInstruction::L(else_label)]);
                        named_variable_map.retain(|&(_, b), _| b <= scope);

                        emit_tactile_statement(code, *else_statement, instructions, max_identifier, max_label, named_variable_map, loop_labels, scope + 1);
                        instructions.push(TACTILEInstruction::L(end_label));
                        named_variable_map.retain(|&(_, b), _| b <= scope);
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
                                                        let expr = emit_tactile_expr(code, init, instructions, max_identifier, max_label, named_variable_map, inner_scope);
                                                        instructions.push(TACTILEInstruction::Copy(expr, var));
                                                }
                                        }
                                        BlockItem::S(astatement) => {
                                                emit_tactile_statement(code, astatement, instructions, max_identifier, max_label, named_variable_map, loop_labels, inner_scope)
                                        }
                                }
                        }
                        named_variable_map.retain(|&(_, f), _| f <= scope);
                }
                AStatement::Break(_) => {
                        instructions.push(TACTILEInstruction::Jump(Label(loop_labels[loop_labels.len() - 1].break_label)));
                }
                AStatement::Continue(_) => {
                        // let mut label = None;
                        // for i in loop_labels.iter().rev() {
                        //         if i.begin == loop_label.0 {
                        //                 label = Some(i.continue_label);
                        //                 break;
                        //         }
                        // }
                        // let label = label.expect("Logic bug. We should have found all breaks and continues outside loops before");
                        // instructions.push(TACTILEInstruction::Jump(Label(label)));
                        instructions.push(TACTILEInstruction::Jump(Label(loop_labels.last().expect("bug").continue_label)));
                }
                AStatement::DoWhile(astatement, aexpression, loop_label) => {
                        let tactile_label = tactilify_loop_label(loop_label, max_label);
                        loop_labels.push(tactile_label);

                        let begin = Label(tactile_label.begin);
                        instructions.push(TACTILEInstruction::L(begin));

                        emit_tactile_statement(code, *astatement, instructions, max_identifier, max_label, named_variable_map, loop_labels, scope + 1);

                        instructions.push(TACTILEInstruction::L(Label(tactile_label.continue_label)));

                        let result = emit_tactile_expr(code, aexpression, instructions, max_identifier, max_label, named_variable_map, scope + 1);

                        instructions.extend([TACTILEInstruction::JumpIfNotZero(result, begin), TACTILEInstruction::L(Label(tactile_label.break_label))]);
                        named_variable_map.retain(|&(_, f), _| f <= scope);
                }
                AStatement::While(aexpression, astatement, loop_label) => {
                        let tactile_label = tactilify_loop_label(loop_label, max_label);
                        loop_labels.push(tactile_label);

                        instructions.push(TACTILEInstruction::L(Label(tactile_label.continue_label)));

                        let result = emit_tactile_expr(code, aexpression, instructions, max_identifier, max_label, named_variable_map, scope + 1);
                        instructions.push(TACTILEInstruction::JumpIfZero(result, Label(tactile_label.break_label)));

                        emit_tactile_statement(code, *astatement, instructions, max_identifier, max_label, named_variable_map, loop_labels, scope + 1);

                        instructions.extend([
                                TACTILEInstruction::Jump(Label(tactile_label.continue_label)),
                                TACTILEInstruction::L(Label(tactile_label.break_label)),
                        ]);
                        named_variable_map.retain(|&(_, f), _| f <= scope);
                }
                AStatement::F(boxed_for, loop_label) => {
                        let For { init, condition, post, body } = *boxed_for;
                        let tactile_label = tactilify_loop_label(loop_label, max_label);
                        loop_labels.push(tactile_label);
                        match init {
                                ForInit::D(Declaration { id, init: Some(initializer) }) => {
                                        let var = emit_tactile_expr(code, AExpression::F(AFactor::Id(id)), instructions, max_identifier, max_label, named_variable_map, scope + 1);
                                        let expr = emit_tactile_expr(code, initializer, instructions, max_identifier, max_label, named_variable_map, scope + 1);
                                        instructions.push(TACTILEInstruction::Copy(expr, var));
                                }
                                ForInit::E(Some(aexpression)) => {
                                        emit_tactile_statement(
                                                code,
                                                AStatement::Expr(aexpression),
                                                instructions,
                                                max_identifier,
                                                max_label,
                                                named_variable_map,
                                                loop_labels,
                                                scope + 1,
                                        );
                                }
                                _ => {}
                        }

                        instructions.push(TACTILEInstruction::L(Label(tactile_label.begin)));
                        if let Some(aexpression) = condition {
                                let value = emit_tactile_expr(code, aexpression, instructions, max_identifier, max_label, named_variable_map, scope + 1);
                                instructions.push(TACTILEInstruction::JumpIfZero(value, Label(tactile_label.break_label)));
                        }
                        () = emit_tactile_statement(code, body, instructions, max_identifier, max_label, named_variable_map, loop_labels, scope + 1);

                        instructions.push(TACTILEInstruction::L(Label(tactile_label.continue_label)));

                        if let Some(aexpression) = post {
                                let _ = emit_tactile_expr(code, aexpression, instructions, max_identifier, max_label, named_variable_map, scope + 1);
                        }

                        instructions.extend([TACTILEInstruction::Jump(Label(tactile_label.begin)), TACTILEInstruction::L(Label(tactile_label.break_label))]);

                        named_variable_map.retain(|&(_, f), _| f <= scope);
                }
        }
}

pub fn tactile(program: Program<SemanticallyAnalyzed>, mut max_label: usize, variable_map: HashMap<(&[u8], usize), Identifier>) -> Program<TACTILE> {
        let code = program.state.code;
        Program {
                operation: program.operation,
                state: TACTILE {
                        program: tactile_program(program.state.program, &code[..], &mut max_label, variable_map),
                        code,
                },
        }
}

fn new_label(max_label: &mut usize) -> Label {
        let temp = Label(*max_label);
        *max_label += 1;
        temp
}
fn new_id(max_identifier: &mut usize) -> Identifier {
        let temp = Identifier(*max_identifier);
        *max_identifier += 1;
        temp
}
