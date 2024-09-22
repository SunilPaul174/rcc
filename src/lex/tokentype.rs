pub static INT: [u8; 3] = [b'i', b'n', b't'];
pub static VOID: [u8; 4] = [b'v', b'o', b'i', b'd'];
pub static RETURN: [u8; 6] = [b'r', b'e', b't', b'u', b'r', b'n'];

#[derive(Debug, Copy, Clone)]
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
