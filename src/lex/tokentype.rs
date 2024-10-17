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
}

#[derive(Debug)]
pub struct Token {
        pub token_type: TokenType,
        pub len: usize,
        pub start: usize,
}
