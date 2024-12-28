use std::{collections::HashMap, hash::BuildHasher};

use crate::{
        parse::nodes::{
                ABlock, AConstant, AExpression, AFactor, AIdentifier, AProgram, AStatement, Binop, BlockItem, BreakType, Conditional, For, ForInit, IfStatement,
                ParseLabel, Switch, Unop, VariableDeclaration,
        },
        semantic_analysis::SemanticallyAnalyzed,
        Program, State,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier(pub usize);

#[derive(Debug, Clone)]
pub struct TACTILE {
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

#[derive(Debug, Clone)]
enum TACTILELabel {
        T(TACTILELoopLabel),
        S(SwitchLabel),
}

#[derive(Debug, Clone)]
struct SwitchLabel {
        label: Label,
        // cases: Vec<Label>,
        // default: Option<Label>,
}

fn tactilify_loop_label(label: ParseLabel, max_label: &mut usize) -> TACTILELoopLabel {
        let break_label = *max_label;
        let continue_label = *max_label + 1;
        let begin = label.0;
        *max_label += 2;
        TACTILELoopLabel {
                begin,
                break_label,
                continue_label,
        }
}

#[derive(Debug, Clone)]
pub struct TACTILEFunction {
        pub identifier: AIdentifier,
        pub instructions: Vec<TACTILEInstruction>,
}

#[derive(Debug, Clone)]
pub struct TACTILEProgram {
        pub functions: Vec<TACTILEFunction>,
}

fn emit_tactile_expr<'b, 'a: 'b, S: BuildHasher>(
        code: &'a [u8],
        value: AExpression,
        instructions: &mut Vec<TACTILEInstruction>,
        max_id: &mut usize,
        max_label: &mut usize,
        identifier_map: &mut HashMap<(&'b [u8], usize), (Identifier, bool), S>,
        scope: usize,
) -> Value {
        match value {
                AExpression::F(AFactor::Constant(n)) => Value::Constant(Constant::A(n)),
                AExpression::F(AFactor::Unop(unop, afactor)) => match unop {
                        Unop::Negate | Unop::Complement | Unop::Not => {
                                let src = emit_tactile_expr(code, AExpression::F(*afactor), instructions, max_id, max_label, identifier_map, scope);
                                let dst = Value::Var(new_id(max_id));
                                instructions.push(TACTILEInstruction::Unary(unop, src, dst));
                                dst
                        }
                        Unop::IncrementPre | Unop::DecrementPre => {
                                let left = emit_tactile_expr(code, AExpression::F(*afactor), instructions, max_id, max_label, identifier_map, scope);
                                instructions.push(TACTILEInstruction::Unary(unop, left, left));
                                left
                        }
                        Unop::IncrementPost | Unop::DecrementPost => {
                                let src = emit_tactile_expr(code, AExpression::F(*afactor), instructions, max_id, max_label, identifier_map, scope);
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

                                let v1 = emit_tactile_expr(code, *left, instructions, max_id, max_label, identifier_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfNotZero(v1, false_label));
                                let v2 = emit_tactile_expr(code, *right, instructions, max_id, max_label, identifier_map, scope);
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

                                let v1 = emit_tactile_expr(code, *left, instructions, max_id, max_label, identifier_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfZero(v1, false_label));
                                let v2 = emit_tactile_expr(code, *right, instructions, max_id, max_label, identifier_map, scope);
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
                                let v1 = emit_tactile_expr(code, *left, instructions, max_id, max_label, identifier_map, scope);
                                let v2 = emit_tactile_expr(code, *right, instructions, max_id, max_label, identifier_map, scope);
                                let dst = Value::Var(new_id(max_id));
                                instructions.push(TACTILEInstruction::Binary(binop, v1, v2, dst));
                                dst
                        }
                },
                AExpression::F(AFactor::Expr(expr)) => emit_tactile_expr(code, *expr, instructions, max_id, max_label, identifier_map, scope),
                AExpression::Assignment(lval, rval) => {
                        let left = emit_tactile_expr(code, *lval, instructions, max_id, max_label, identifier_map, scope);
                        let right = emit_tactile_expr(code, *rval, instructions, max_id, max_label, identifier_map, scope);
                        instructions.push(TACTILEInstruction::Copy(right, left));
                        left
                }
                AExpression::F(AFactor::Id(id)) => {
                        let AIdentifier { start, len } = id;
                        let name = &code[start..start + len];
                        let mut potential = identifier_map.keys().filter(|(f, _)| *f == name).peekable();

                        if potential.peek().is_none() {
                                let entered = identifier_map.entry((name, scope)).insert_entry((new_id(max_id), false));
                                return Value::Var(entered.get().0);
                        }

                        let max = potential.max_by_key(|f| f.1).unwrap();

                        Value::Var(identifier_map.get(max).unwrap().0)
                }
                AExpression::OpAssignment(binop, left, right) => {
                        let left = emit_tactile_expr(code, *left, instructions, max_id, max_label, identifier_map, scope);
                        let right = emit_tactile_expr(code, *right, instructions, max_id, max_label, identifier_map, scope);

                        instructions.push(TACTILEInstruction::Binary(binop, left, right, left));
                        left
                }
                AExpression::C(Conditional { condition, True, False }) => {
                        let end_label = new_label(max_label);
                        let else_label = new_label(max_label);

                        let result = Value::Var(new_id(max_id));

                        let c = emit_tactile_expr(code, *condition, instructions, max_id, max_label, identifier_map, scope);
                        instructions.push(TACTILEInstruction::JumpIfZero(c, else_label));

                        let val1 = emit_tactile_expr(code, *True, instructions, max_id, max_label, identifier_map, scope);
                        instructions.extend_from_slice(&[
                                TACTILEInstruction::Copy(val1, result),
                                TACTILEInstruction::Jump(end_label),
                                TACTILEInstruction::L(else_label),
                        ]);

                        let val2 = emit_tactile_expr(code, *False, instructions, max_id, max_label, identifier_map, scope);
                        instructions.extend_from_slice(&[TACTILEInstruction::Copy(val2, result), TACTILEInstruction::L(end_label)]);

                        result
                }
                AExpression::FunctionCall(aidentifier, vec) => todo!(),
        }
}

fn tactile_program<'b, 'a: 'b, S: BuildHasher>(
        program: AProgram,
        code: &'a [u8],
        max_label: &mut usize,
        mut identifier_map: HashMap<(&'b [u8], usize), (Identifier, bool), S>,
) -> TACTILEProgram {
        let value = program.functions;
        let mut global_max_identifier = 1;
        let mut loop_labels = vec![];
        let mut functions = vec![];
        let scope = 0;

        for i in value {
                if let Some(body) = i.body {
                        for j in body.0 {
                                let mut instructions = vec![];
                                tactile_block_item(
                                        j,
                                        code,
                                        &mut instructions,
                                        &mut global_max_identifier,
                                        max_label,
                                        &mut identifier_map,
                                        scope,
                                        &mut loop_labels,
                                );
                                instructions.push(TACTILEInstruction::Return(Value::Constant(Constant::S(0))));
                                functions.push(TACTILEFunction {
                                        identifier: i.name,
                                        instructions,
                                });
                        }
                }
        }

        TACTILEProgram { functions }
}

fn tactile_block_item<'b, 'a: 'b, S: BuildHasher>(
        block_item: BlockItem,
        code: &'a [u8],
        instructions: &mut Vec<TACTILEInstruction>,
        global_max_identifier: &mut usize,
        max_label: &mut usize,
        identifier_map: &mut HashMap<(&'b [u8], usize), (Identifier, bool), S>,
        scope: usize,
        loop_labels: &mut Vec<TACTILELabel>,
) {
        match block_item {
                BlockItem::D(declaration) => {
                        // if let Some(init) = declaration.init {
                        //         let var = emit_tactile_expr(
                        //                 code,
                        //                 AExpression::F(AFactor::Id(declaration.id)),
                        //                 instructions,
                        //                 global_max_identifier,
                        //                 max_label,
                        //                 identifier_map,
                        //                 scope,
                        //         );
                        //         let expr = emit_tactile_expr(code, init, instructions, global_max_identifier, max_label, identifier_map, scope);
                        //         instructions.push(TACTILEInstruction::Copy(expr, var));
                        // }
                        todo!()
                }
                BlockItem::S(astatement) => emit_tactile_statement(code, astatement, instructions, global_max_identifier, max_label, identifier_map, loop_labels, scope),
        }
}

fn emit_tactile_statement<'b, 'a: 'b, S: BuildHasher>(
        code: &'a [u8],
        value: AStatement,
        instructions: &mut Vec<TACTILEInstruction>,
        max_id: &mut usize,
        max_label: &mut usize,
        identifier_map: &mut HashMap<(&'b [u8], usize), (Identifier, bool), S>,
        labels: &mut Vec<TACTILELabel>,
        scope: usize,
) {
        match value {
                AStatement::Return(aexpression) => {
                        let val = emit_tactile_expr(code, aexpression, instructions, max_id, max_label, identifier_map, scope);
                        instructions.push(TACTILEInstruction::Return(val));
                }
                AStatement::Expr(aexpression) => {
                        let _ = emit_tactile_expr(code, aexpression, instructions, max_id, max_label, identifier_map, scope);
                }
                AStatement::Nul => {}
                AStatement::I(IfStatement {
                        condition,
                        then,
                        Else: else_statement,
                }) => {
                        if else_statement.is_none() {
                                let end = new_label(max_label);

                                let c = emit_tactile_expr(code, condition, instructions, max_id, max_label, identifier_map, scope);
                                instructions.push(TACTILEInstruction::JumpIfZero(c, end));

                                emit_tactile_statement(code, *then, instructions, max_id, max_label, identifier_map, labels, scope + 1);

                                identifier_map.retain(|&(_, f), _| f <= scope);

                                instructions.push(TACTILEInstruction::L(end));

                                return;
                        }

                        let else_statement = else_statement.unwrap();
                        let end_label = new_label(max_label);
                        let else_label = new_label(max_label);

                        let c = emit_tactile_expr(code, condition, instructions, max_id, max_label, identifier_map, scope);
                        instructions.push(TACTILEInstruction::JumpIfZero(c, else_label));

                        emit_tactile_statement(code, *then, instructions, max_id, max_label, identifier_map, labels, scope + 1);
                        instructions.extend_from_slice(&[TACTILEInstruction::Jump(end_label), TACTILEInstruction::L(else_label)]);
                        identifier_map.retain(|&(_, b), _| b <= scope);

                        emit_tactile_statement(code, *else_statement, instructions, max_id, max_label, identifier_map, labels, scope + 1);
                        instructions.push(TACTILEInstruction::L(end_label));
                        identifier_map.retain(|&(_, b), _| b <= scope);
                }
                AStatement::Compound(ABlock(block)) => {
                        let inner_scope = scope + 1;
                        for i in block {
                                match i {
                                        BlockItem::D(declaration) => {
                                                // if let Some(init) = declaration.init {
                                                //         let var = emit_tactile_expr(
                                                //                 code,
                                                //                 AExpression::F(AFactor::Id(declaration.id)),
                                                //                 instructions,
                                                //                 max_id,
                                                //                 max_label,
                                                //                 identifier_map,
                                                //                 inner_scope,
                                                //         );
                                                //         let expr = emit_tactile_expr(code, init, instructions, max_id, max_label, identifier_map, inner_scope);
                                                //         instructions.push(TACTILEInstruction::Copy(expr, var));
                                                // }
                                                todo!()
                                        }
                                        BlockItem::S(astatement) => {
                                                emit_tactile_statement(code, astatement, instructions, max_id, max_label, identifier_map, labels, inner_scope);
                                        }
                                }
                        }
                        identifier_map.retain(|&(_, f), _| f <= scope);
                }
                AStatement::Break(_, breaktype) => match breaktype {
                        BreakType::Loop => {
                                let TACTILELabel::T(last_label) = labels[labels.len() - 1] else {
                                        panic!("logic bug")
                                };
                                instructions.push(TACTILEInstruction::Jump(Label(last_label.break_label)))
                        }
                        BreakType::Switch => {
                                let TACTILELabel::S(last_label) = &labels[labels.len() - 1] else {
                                        panic!("logic bug")
                                };

                                instructions.push(TACTILEInstruction::Jump(last_label.label));
                        }
                },
                AStatement::Continue(_) => {
                        let TACTILELabel::T(last_label) = labels[labels.len() - 1] else {
                                panic!("logic bug")
                        };
                        instructions.push(TACTILEInstruction::Jump(Label(last_label.continue_label)));
                }
                AStatement::DoWhile(astatement, aexpression, loop_label) => {
                        let tactile_label = tactilify_loop_label(loop_label, max_label);
                        labels.push(TACTILELabel::T(tactile_label));

                        let begin = Label(tactile_label.begin);
                        instructions.push(TACTILEInstruction::L(begin));

                        emit_tactile_statement(code, *astatement, instructions, max_id, max_label, identifier_map, labels, scope + 1);

                        instructions.push(TACTILEInstruction::L(Label(tactile_label.continue_label)));

                        let result = emit_tactile_expr(code, aexpression, instructions, max_id, max_label, identifier_map, scope + 1);

                        instructions.extend([
                                TACTILEInstruction::JumpIfNotZero(result, begin),
                                TACTILEInstruction::L(Label(tactile_label.break_label)),
                        ]);
                        identifier_map.retain(|&(_, f), _| f <= scope);
                }
                AStatement::While(aexpression, astatement, loop_label) => {
                        let tactile_label = tactilify_loop_label(loop_label, max_label);
                        labels.push(TACTILELabel::T(tactile_label));

                        instructions.push(TACTILEInstruction::L(Label(tactile_label.continue_label)));

                        let result = emit_tactile_expr(code, aexpression, instructions, max_id, max_label, identifier_map, scope + 1);
                        instructions.push(TACTILEInstruction::JumpIfZero(result, Label(tactile_label.break_label)));

                        emit_tactile_statement(code, *astatement, instructions, max_id, max_label, identifier_map, labels, scope + 1);

                        instructions.extend([
                                TACTILEInstruction::Jump(Label(tactile_label.continue_label)),
                                TACTILEInstruction::L(Label(tactile_label.break_label)),
                        ]);
                        identifier_map.retain(|&(_, f), _| f <= scope);
                }
                AStatement::F(boxed_for, loop_label) => {
                        let For { init, condition, post, body } = *boxed_for;
                        let tactile_label = tactilify_loop_label(loop_label, max_label);
                        labels.push(TACTILELabel::T(tactile_label));
                        match init {
                                ForInit::D(VariableDeclaration { id, init: Some(initializer) }) => {
                                        let var = emit_tactile_expr(code, AExpression::F(AFactor::Id(id)), instructions, max_id, max_label, identifier_map, scope + 1);
                                        let expr = emit_tactile_expr(code, initializer, instructions, max_id, max_label, identifier_map, scope + 1);
                                        instructions.push(TACTILEInstruction::Copy(expr, var));
                                }
                                ForInit::E(Some(aexpression)) => {
                                        let expr = AStatement::Expr(aexpression);
                                        emit_tactile_statement(code, expr, instructions, max_id, max_label, identifier_map, labels, scope + 1);
                                }

                                _ => {}
                        }

                        instructions.push(TACTILEInstruction::L(Label(tactile_label.begin)));
                        if let Some(aexpression) = condition {
                                let value = emit_tactile_expr(code, aexpression, instructions, max_id, max_label, identifier_map, scope + 1);
                                instructions.push(TACTILEInstruction::JumpIfZero(value, Label(tactile_label.break_label)));
                        }
                        () = emit_tactile_statement(code, body, instructions, max_id, max_label, identifier_map, labels, scope + 1);

                        instructions.push(TACTILEInstruction::L(Label(tactile_label.continue_label)));

                        if let Some(aexpression) = post {
                                let _ = emit_tactile_expr(code, aexpression, instructions, max_id, max_label, identifier_map, scope + 1);
                        }

                        instructions.extend([
                                TACTILEInstruction::Jump(Label(tactile_label.begin)),
                                TACTILEInstruction::L(Label(tactile_label.break_label)),
                        ]);

                        identifier_map.retain(|&(_, f), _| f <= scope);
                }
                AStatement::S(switch) => {
                        let Switch { value, cases, default, label } = switch;

                        if cases.is_empty() && default.is_none() {
                                return;
                        }

                        let mut case_labels = Vec::with_capacity(cases.len() - 1);

                        for _ in 0..cases.len() {
                                case_labels.push(new_label(max_id));
                        }

                        // let mut default_label = None;
                        let mut pot_def = None;
                        if let Some(default) = default {
                                // default_label = Some(new_label(max_label));
                                pot_def = Some(default);
                        }

                        let break_label = Label(label.0);

                        labels.push(TACTILELabel::S(SwitchLabel {
                                // cases: case_labels.clone(),
                                // default: default_label,
                                label: break_label,
                        }));

                        let value = emit_tactile_expr(code, value, instructions, max_id, max_label, identifier_map, scope);

                        for (idx, (constant, statements)) in cases.into_iter().enumerate() {
                                let curr_const = Value::Constant(Constant::A(constant));
                                let dst = Value::Var(new_id(max_id));

                                instructions.extend([
                                        TACTILEInstruction::Binary(Binop::EqualTo, value, curr_const, dst),
                                        TACTILEInstruction::JumpIfZero(dst, case_labels[idx]),
                                ]);

                                for i in statements {
                                        () = emit_tactile_statement(code, i, instructions, max_id, max_label, identifier_map, labels, scope);
                                }

                                if idx > 0 {
                                        instructions.push(TACTILEInstruction::L(case_labels[idx - 1]));
                                }
                        }
                        if let Some(case_label) = case_labels.last() {
                                instructions.push(TACTILEInstruction::L(*case_label));
                        }

                        if let Some(default) = pot_def {
                                let default = *default;
                                () = emit_tactile_statement(code, default, instructions, max_id, max_label, identifier_map, labels, scope);
                        }

                        instructions.push(TACTILEInstruction::L(break_label));
                }
        }
}

pub fn tactile<S: BuildHasher>(
        program: Program<SemanticallyAnalyzed>,
        mut max_label: usize,
        identifier_map: HashMap<(&[u8], usize), (Identifier, bool), S>,
        code: &[u8],
) -> Program<TACTILE> {
        Program {
                operation: program.operation,
                state: TACTILE {
                        program: tactile_program(program.state.program, code, &mut max_label, identifier_map),
                },
                obj: program.obj,
        }
}

fn new_label(max_label: &mut usize) -> Label {
        let temp = Label(*max_label);
        *max_label += 1;
        temp
}
fn new_id(max_id: &mut usize) -> Identifier {
        let temp = Identifier(*max_id);
        *max_id += 1;
        temp
}
