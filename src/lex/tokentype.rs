use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
        Int,
        Void,
        Return,
        Identifier,
        Constant,
        OpenParen,
        CloseParen,
        OpenBrace,
        CloseBrace,
        SemiColon,
        Minus,
        Tilde,
        Plus,
        Asterisk,
        ForwardSlash,
        Percent,
        Decrement,
        Equal,
        BitwiseAnd,
        LogicalAnd,
        BitwiseOr,
        LogicalOr,
        BitwiseXOr,
        LeftShift,
        RightShift,
        LessThan,
        MoreThan,
        LessThanOrEqual,
        MoreThanOrEqual,
        EqualTo,
        Not,
        NotEqualTo,
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
        pub token_type: TokenType,
        pub len: usize,
        pub start: usize,
}

impl Display for Token {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}, starting at {}, with length {}", self.token_type, self.start, self.len) }
}
impl Display for TokenType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
