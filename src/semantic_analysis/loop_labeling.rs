use crate::parse::nodes::{ABlock, AProgram, AStatement, BlockItem, ParseLabel, Switch};

use super::Error;

pub(super) fn label_loops(program: &mut AProgram) -> Result<usize, Error> {
        let mut max_label = ParseLabel(0);
        for i in &mut program.functions {
                if let Some(body) = &mut i.body {
                        for j in &mut body.0 {
                                match j {
                                        BlockItem::D(_) => {}
                                        BlockItem::S(astatement) => label_statement(astatement, None, &mut max_label)?,
                                }
                        }
                }
        }

        Ok(max_label.0)
}

fn label_statement(statement: &mut AStatement, curr_label: Option<ParseLabel>, max_label: &mut ParseLabel) -> Result<(), Error> {
        match statement {
                AStatement::Break(..) | AStatement::Continue(_) => {
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
                AStatement::S(switch) => {
                        let switch_label = new_label(max_label);
                        let Switch { cases, default, label, .. } = switch;
                        *label = switch_label;

                        for i in cases {
                                for j in &mut i.1 {
                                        annotate(j, switch_label);
                                }
                        }

                        if let Some(default) = default {
                                annotate(default, switch_label);
                        }
                }
                AStatement::Return(_) | AStatement::Expr(_) | AStatement::Nul => {}
        }

        Ok(())
}

fn new_label(max_label: &mut ParseLabel) -> ParseLabel {
        let temp = ParseLabel(max_label.0);
        max_label.0 += 1;
        temp
}

fn annotate(statement: &mut AStatement, curr_label: ParseLabel) {
        match statement {
                AStatement::Break(label, _) | AStatement::Continue(label) => {
                        *label = curr_label;
                }
                _ => {}
        }
}
