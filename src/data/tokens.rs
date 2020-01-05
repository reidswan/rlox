use super::literals::Literal;
use super::meta::MetaContainer;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Token {
    // single char
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Question,
    Colon,

    // One or two chars
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Lesser,
    LesserEqual,

    // Literals
    Literal(Literal),

    // Keywords
    And,
    Class,
    Else,
    Fun,
    For,
    If,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,

    EOF,
}

impl Token {
    pub fn is_value(&self) -> bool {
        use Token::*;
        match self {
            Literal(_) => true,
            _ => false,
        }
    }
    pub fn is_operator(&self) -> bool {
        use Token::*;
        match self {
            Bang | BangEqual | Equal | EqualEqual | Greater | GreaterEqual => true,
            Lesser | LesserEqual | Star | Slash | Plus | Minus => true,
            _ => false,
        }
    }
}


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Token::*;
        write!(
            f,
            "{}",
            match self {
                LeftParen => "(".to_owned(),
                RightParen => ")".to_owned(),
                LeftBrace => "{".to_owned(),
                RightBrace => "}".to_owned(),
                Comma => ",".to_owned(),
                Dot => ".".to_owned(),
                Minus => "-".to_owned(),
                Plus => "+".to_owned(),
                Semicolon => ";".to_owned(),
                Slash => "/".to_owned(),
                Star => "*".to_owned(),
                Question => "?".to_owned(),
                Colon => ":".to_owned(),

                // One or two chars
                Bang => "!".to_owned(),
                BangEqual => "!=".to_owned(),
                Equal => "=".to_owned(),
                EqualEqual => "==".to_owned(),
                Greater => ">".to_owned(),
                GreaterEqual => ">=".to_owned(),
                Lesser => "<".to_owned(),
                LesserEqual => "<=".to_owned(),

                Literal(l) => format!("{}", l),

                // Keywords
                And => "and".to_owned(),
                Class => "class".to_owned(),
                Else => "else".to_owned(),
                Fun => "fun".to_owned(),
                For => "for".to_owned(),
                If => "if".to_owned(),
                Or => "or".to_owned(),
                Print => "print".to_owned(),
                Return => "return".to_owned(),
                Super => "super".to_owned(),
                This => "this".to_owned(),
                Var => "var".to_owned(),
                While => "while".to_owned(),

                EOF => "EOF".to_owned(),
            }
        )
    }
}

pub type TokenMeta = MetaContainer<Token>;
