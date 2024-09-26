pub static INT: &[u8; 3] = b"int";
pub static VOID: &[u8; 3] = b"vod";
pub static RETURN: &[u8; 3] = b"ren";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
        // usizes are lengths
        Identifier(usize),
        Constant(usize),
        KeywordInt,
        KeywordVoid,
        KeywordReturn,
        OpenBrace,
        CloseBrace,
        OpenParanthesis,
        CloseParanthesis,
        SemiColon,
}
