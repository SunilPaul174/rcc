use crate::parse::nodes::{ABlock, AProgram, AStatement, BlockItem, LoopLabel};

use super::Error;

pub(super) fn label_loops(program: &mut AProgram) -> Result<usize, Error> {
        let mut max_label = LoopLabel(0);
        for i in &mut program.function.function_body.0 {
                match i {
                        BlockItem::D(_) => {}
                        BlockItem::S(astatement) => label_statement(astatement, None, &mut max_label)?,
                }
        }

        Ok(max_label.0)
}

fn label_statement(statement: &mut AStatement, curr_label: Option<LoopLabel>, max_label: &mut LoopLabel) -> Result<(), Error> {
        match statement {
                AStatement::Break(_) | AStatement::Continue(_) => {
                        let Some(label) = curr_label else {
                                return Err(Error::BreakOutsideLoop(statement.clone()));
                        };
                        annotate(statement, label);
                }
                AStatement::While(_, astatement, loop_label) | AStatement::DoWhile(astatement, _, loop_label) => {
                        let new_label = new_label(max_label);
                        *loop_label = new_label;
                        label_statement(astatement, Some(new_label), max_label)?;
                }
                AStatement::F(boxed_for, loop_label) => {
                        let new_label = new_label(max_label);
                        *loop_label = new_label;
                        label_statement(&mut boxed_for.body, Some(new_label), max_label)?;
                }
                AStatement::I(if_statement) => {
                        label_statement(&mut if_statement.then, curr_label, max_label)?;
                        if if_statement.Else.is_some() {
                                label_statement(if_statement.Else.as_mut().unwrap(), curr_label, max_label)?;
                        }
                }
                AStatement::Compound(ABlock(vec)) => {
                        for i in vec {
                                match i {
                                        BlockItem::D(_) => {}
                                        BlockItem::S(astatement) => label_statement(astatement, curr_label, max_label)?,
                                }
                        }
                }
                AStatement::Return(_) | AStatement::Expr(_) | AStatement::Nul => {}
        }

        Ok(())
}

fn new_label(max_label: &mut LoopLabel) -> LoopLabel {
        let temp = LoopLabel(max_label.0);
        max_label.0 += 1;
        temp
}

fn annotate(statement: &mut AStatement, curr_label: LoopLabel) {
        match statement {
                AStatement::Break(label) | AStatement::Continue(label) => {
                        *label = curr_label;
                }
                _ => {}
        }
}
