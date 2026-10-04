use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Out,
    In,
    Shape,
    From,
    Where,
    Select,
    And,
    Or,
    Not,
    True,
    False,
    Null,

    // Identifiers & Literals
    Ident(String),
    StringLit(String),
    NumberLit(f64),

    // Symbols & Operators
    Dot,            // .
    Colon,          // :
    Comma,          // ,
    Equals,         // =
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    LParen,         // (
    RParen,         // )
    DoubleQuestion, // ??
    Plus,           // +
    EqEq,           // ==
    NotEq,          // !=
    Lt,             // <
    LtEq,           // <=
    Gt,             // >
    GtEq,           // >=
    Bang,           // !

    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Out => write!(f, "'out'"),
            Token::In => write!(f, "'in'"),
            Token::Shape => write!(f, "'shape'"),
            Token::From => write!(f, "'from'"),
            Token::Where => write!(f, "'where'"),
            Token::Select => write!(f, "'select'"),
            Token::And => write!(f, "'and'"),
            Token::Or => write!(f, "'or'"),
            Token::Not => write!(f, "'not'"),
            Token::True => write!(f, "'true'"),
            Token::False => write!(f, "'false'"),
            Token::Null => write!(f, "'null'"),
            Token::Ident(s) => write!(f, "identifier '{}'", s),
            Token::StringLit(s) => write!(f, "string \"{}\"", s),
            Token::NumberLit(n) => write!(f, "number {}", n),
            Token::Dot => write!(f, "'.'"),
            Token::Colon => write!(f, "':'"),
            Token::Comma => write!(f, "','"),
            Token::Equals => write!(f, "'='"),
            Token::LBrace => write!(f, "'{{'"),
            Token::RBrace => write!(f, "'}}'"),
            Token::LBracket => write!(f, "'['"),
            Token::RBracket => write!(f, "']'"),
            Token::LParen => write!(f, "'('"),
            Token::RParen => write!(f, "')'"),
            Token::DoubleQuestion => write!(f, "'??'"),
            Token::Plus => write!(f, "'+'"),
            Token::EqEq => write!(f, "'=='"),
            Token::NotEq => write!(f, "'!='"),
            Token::Lt => write!(f, "'<'"),
            Token::LtEq => write!(f, "'<='"),
            Token::Gt => write!(f, "'>'"),
            Token::GtEq => write!(f, "'>='"),
            Token::Bang => write!(f, "'!'"),
            Token::Eof => write!(f, "end of file"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}
