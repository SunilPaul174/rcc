use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct AProgram {
        pub function: AFunction,
}

#[derive(Debug, Clone)]
pub struct AFunction {
        pub identifier: AIdentifier,
        pub function_body: ABlock,
}

#[derive(Debug, Clone)]
pub struct ABlock(pub Vec<BlockItem>);

#[derive(Debug, Clone, Copy)]
pub struct AIdentifier {
        pub start: usize,
        pub len: usize,
}
#[derive(Debug, Clone)]
pub enum BlockItem {
        D(Declaration),
        S(AStatement),
}
#[derive(Debug, Clone)]
pub struct Declaration {
        pub id: AIdentifier,
        pub init: Option<AExpression>,
}
#[derive(Debug, Clone)]
pub enum AStatement {
        Return(AExpression),
        Expr(AExpression),
        I(IfStatement),
        Compound(ABlock),
        Nul,
        Break(ParseLabel, BreakType),
        Continue(ParseLabel),
        While(AExpression, Box<AStatement>, ParseLabel),
        DoWhile(Box<AStatement>, AExpression, ParseLabel),
        F(Box<For>, ParseLabel),
        S(Switch),
}

#[derive(Debug, Clone, Copy)]
pub enum BreakType {
        Loop,
        Switch,
}
#[derive(Debug, Clone, Copy)]
pub enum LoopSwitchOrNone {
        Loop,
        Switch,
        Neither,
}

#[derive(Debug, Clone)]
pub struct Switch {
        pub value: AExpression,
        pub cases: Vec<(AConstant, Vec<AStatement>)>,
        pub default: Option<Box<AStatement>>,
        pub label: ParseLabel,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseLabel(pub usize);

#[derive(Debug, Clone)]
pub struct For {
        pub init: ForInit,
        pub condition: Option<AExpression>,
        pub post: Option<AExpression>,
        pub body: AStatement,
}
#[derive(Debug, Clone)]
pub enum ForInit {
        D(Declaration),
        E(Option<AExpression>),
}

#[derive(Debug, Clone)]
pub struct IfStatement {
        pub condition: AExpression,
        pub then: Box<AStatement>,
        pub Else: Option<Box<AStatement>>,
}
#[derive(Debug, Clone)]
pub enum AFactor {
        Constant(AConstant),
        Unop(Unop, Box<AFactor>),
        Expr(Box<AExpression>),
        Id(AIdentifier),
}

#[derive(Debug, Clone)]
pub enum AExpression {
        F(AFactor),
        BinOp(Binop, Box<AExpression>, Box<AExpression>),
        Assignment(Box<AExpression>, Box<AExpression>),
        OpAssignment(Binop, Box<AExpression>, Box<AExpression>),
        C(Conditional),
}
#[derive(Debug, Clone)]
pub struct Conditional {
        pub condition: Box<AExpression>,
        pub True: Box<AExpression>,
        pub False: Box<AExpression>,
}
impl Display for AExpression {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                        AExpression::F(afactor) => write!(f, "{afactor:?}",),
                        AExpression::BinOp(..) => write!(f, "{self:?}"),
                        AExpression::Assignment(left, right) => write!(f, "left: {left:?}, right: {right:?}"),
                        AExpression::C(Conditional { condition, True, False }) => write!(f, "condition: {condition}, True: {True}, False: {False}"),
                        AExpression::OpAssignment(binop, left, right) => write!(f, "operator: {binop:?}, left: {left}, right: {right}"),
                }
        }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unop {
        Negate,
        Complement,
        // !
        Not,
        IncrementPre,
        IncrementPost,
        DecrementPre,
        DecrementPost,
}
#[derive(Debug, Clone, Copy)]
pub struct AConstant {
        pub start: usize,
        pub len: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Binop {
        Add,
        AddAssign,
        Subtract,
        SubtractAssign,
        Multiply,
        MultiplyAssign,
        Divide,
        DivideAssign,
        Remainder,
        RemainderAssign,
        LeftShift,
        LeftShiftAssign,
        RightShift,
        RightShiftAssign,
        BitwiseAnd,
        BitwiseAndAssign,
        LogicalAnd,
        LogicalAndAssign,
        BitwiseOr,
        BitwiseOrAssign,
        LogicalOr,
        LogicalOrAssign,
        BitwiseXOr,
        BitwiseXOrAssign,
        EqualTo,
        NotEqualTo,
        LessThan,
        LessThanOrEqual,
        MoreThan,
        MoreThanOrEqual,
        Equal,
        Ternary,
}
