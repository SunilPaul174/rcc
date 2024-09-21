#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
        // usizes are lengths
        Identifier(usize),
        Constant(usize),
        KeywordAuto,
        KeywordBreak,
        KeywordCase,
        KeywordChar,
        KeywordConst,
        KeywordContinue,
        KeywordDefault,
        KeywordDo,
        KeywordDouble,
        KeywordElse,
        KeywordEnum,
        KeywordExtern,
        KeywordFloat,
        KeywordFor,
        KeywordGoto,
        KeywordIf,
        KeywordLong,
        KeywordRegister,
        KeywordReturn,
        KeywordShort,
        KeywordSigned,
        KeywordSizeof,
        KeywordStatic,
        KeywordStruct,
        KeywordSwitch,
        KeywordTypedef,
        KeywordUnion,
        KeywordUnsigned,
        KeywordVoid,
        KeywordVolatile,
        KeywordWhile,
        KeywordInt,
        OpenBrace,
        CloseBrace,
        OpenParanthesis,
        CloseParanthesis,
        SemiColon,
}

impl TokenType {
        pub fn len(&self) -> usize {
                match self {
                        TokenType::KeywordAuto => {}
                        TokenType::KeywordBreak => {}
                        TokenType::KeywordCase => {}
                        TokenType::KeywordChar => {}
                        TokenType::KeywordConst => {}
                        TokenType::KeywordContinue => {}
                        TokenType::KeywordDefault => {}
                        TokenType::KeywordDo => {}
                        TokenType::KeywordDouble => {}
                        TokenType::KeywordElse => {}
                        TokenType::KeywordEnum => {}
                        TokenType::KeywordExtern => {}
                        TokenType::KeywordFloat => {}
                        TokenType::KeywordFor => {}
                        TokenType::KeywordGoto => {}
                        TokenType::KeywordIf => {}
                        TokenType::KeywordLong => {}
                        TokenType::KeywordRegister => {}
                        TokenType::KeywordReturn => {}
                        TokenType::KeywordShort => {}
                        TokenType::KeywordSigned => {}
                        TokenType::KeywordSizeof => {}
                        TokenType::KeywordStatic => {}
                        TokenType::KeywordStruct => {}
                        TokenType::KeywordSwitch => {}
                        TokenType::KeywordTypedef => {}
                        TokenType::KeywordUnion => {}
                        TokenType::KeywordUnsigned => {}
                        TokenType::KeywordVoid => {}
                        TokenType::KeywordVolatile => {}
                        TokenType::KeywordWhile => {}
                        TokenType::KeywordInt => {}
                        TokenType::OpenBrace => {}
                        TokenType::CloseBrace => {}
                        TokenType::OpenParanthesis => {}
                        TokenType::CloseParanthesis => {}
                        TokenType::SemiColon => {}
                        _ => {}
                }

                todo!()
        }
}
