use crate::errors::{ErrorData, LoxError};
use crate::data::literals::Literal;
use crate::data::tokens::{Token, TokenMeta};
use lazy_static::*;
use std::collections::HashMap;

lazy_static! {
    static ref RESERVED_WORDS: HashMap<&'static str, Token> = {
        let mut m = HashMap::new();
        m.insert("and", Token::And);
        m.insert("class", Token::Class);
        m.insert("else", Token::Else);
        m.insert("false", Token::Literal(Literal::False));
        m.insert("fun", Token::Fun);
        m.insert("for", Token::For);
        m.insert("if", Token::If);
        m.insert("nil", Token::Literal(Literal::Nil));
        m.insert("or", Token::Or);
        m.insert("print", Token::Print);
        m.insert("return", Token::Return);
        m.insert("super", Token::Super);
        m.insert("this", Token::This);
        m.insert("true", Token::Literal(Literal::True));
        m.insert("var", Token::Var);
        m.insert("while", Token::While);
        m
    };
}

pub struct Scanner {
    src: Vec<char>,
    current: usize,
    line_no: usize,
}

macro_rules! if_peek_eq {
    ($self:ident,$c:literal,$option_true:expr,$option_false:expr) => {
        if let Some($c) = $self.peek_char() {
            $self.next_char(); // consume the peeked char
            $option_true
        } else {
            $option_false
        }
    };
}

impl Scanner {
    pub fn new(src: &str) -> Self {
        Scanner {
            src: src.chars().collect(),
            current: 0,
            line_no: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<TokenMeta>, Vec<LoxError>> {
        let mut errors = vec![];
        let mut tokens = vec![];

        while let Some(r) = self.next_token() {
            match r {
                Ok(tok) => tokens.push(tok),
                Err(e) => errors.push(e),
            }
        }

        if errors.is_empty() {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    fn is_end(&self) -> bool {
        self.current >= self.src.len()
    }

    /// get the next character, consuming it
    fn next_char(&mut self) -> Option<char> {
        if self.is_end() {
            None
        } else {
            let result = self.src[self.current];
            self.current += 1;
            if result == '\n' {
                self.line_no += 1
            }
            Some(result)
        }
    }

    /// Get the next character without consuming it
    fn peek_char(&self) -> Option<char> {
        if self.is_end() {
            None
        } else {
            Some(self.src[self.current])
        }
    }

    /// Get the character after next without consuming it
    fn peek_char_after(&self) -> Option<char> {
        if self.current + 1 >= self.src.len() {
            None
        } else {
            Some(self.src[self.current + 1])
        }
    }

    fn next_token(&mut self) -> Option<Result<TokenMeta, LoxError>> {
        use Token::*;
        let token = match self.next_char()? {
            '(' => LeftParen,
            ')' => RightParen,
            '{' => LeftBrace,
            '}' => RightBrace,
            ',' => Comma,
            '.' => Dot,
            '-' => Minus,
            '+' => Plus,
            ';' => Semicolon,
            '*' => Star,
            '?' => Question,
            ':' => Colon,
            '!' => if_peek_eq!(self, '=', BangEqual, Bang),
            '=' => if_peek_eq!(self, '=', EqualEqual, Equal),
            '<' => if_peek_eq!(self, '=', LesserEqual, Lesser),
            '>' => if_peek_eq!(self, '=', GreaterEqual, Greater),
            '/' if self.peek_char() == Some('/') => {
                // consume until end of line
                self.current = self.find_next('\n').unwrap_or(self.src.len());
                return self.next_token();
            }
            '/' => Slash,
            '\r' | ' ' | '\t' => {
                while let Some(c) = self.peek_char() {
                    if c == '\r' || c == ' ' || c == '\t' {
                        self.next_char();
                    } else {
                        break;
                    }
                }
                return self.next_token();
            }
            '\n' => return self.next_token(),
            '"' => return Some(self.match_string()),
            c if c.is_ascii_digit() => match self.match_numeric() {
                Ok(tok) => tok,
                Err(e) => return Some(Err(e)),
            },
            c if can_start_identifier(c) => match self.match_identifier() {
                Ok(tok) => tok,
                Err(e) => return Some(Err(e)),
            },
            c => {
                return Some(Err(LoxError::ScannerError(ErrorData {
                    message: format!("Unidentified character {}", c),
                    line_no: self.line_no,
                    location: String::new(),
                })))
            }
        };
        Some(Ok(TokenMeta::new(token, self.line_no)))
    }

    /// Get the index of the next occurrence of c if it exists in the string
    fn find_next(&self, c: char) -> Option<usize> {
        let mut curr = self.current + 1;
        while curr < self.src.len() {
            if self.src[curr] == c {
                return Some(curr);
            }
            curr += 1
        }
        None
    }

    /// match a literal string (excluding leading '"')
    /// note: returns a TokenMeta because the line number can change while scanning
    fn match_string(&mut self) -> Result<TokenMeta, LoxError> {
        let start_line = self.line_no;
        let mut string = String::new();
        let mut is_escape = false;
        let mut ended = false;
        while let Some(c) = self.next_char() {
            if is_escape {
                string.push(match c {
                    't' => '\t',
                    '\\' => '\\',
                    'n' => '\n',
                    'r' => '\r',
                    '\'' => '\'',
                    '"' => '"',
                    '\n' => continue,
                    _ => {
                        return Err(LoxError::ScannerError(ErrorData {
                            message: format!("Invalid escape char: {}", c),
                            line_no: self.line_no,
                            location: String::from(""),
                        }))
                    }
                });
                is_escape = false;
            } else {
                match c {
                    '\\' => is_escape = true,
                    '"' => {
                        ended = true;
                        break;
                    }
                    _ => string.push(c),
                }
            }
        }

        if !ended {
            // we ran out of characters before reaching closing '"'
            Err(LoxError::ScannerError(ErrorData {
                message: String::from("Unterminated string literal"),
                line_no: self.line_no,
                location: String::new(),
            }))
        } else {
            Ok(TokenMeta::new(
                Token::Literal(Literal::StringT(string)),
                start_line,
            ))
        }
    }

    fn match_numeric(&mut self) -> Result<Token, LoxError> {
        let start = self.current - 1;
        let mut is_int = true;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }

        if let Some('.') = self.peek_char() {
            if self
                .peek_char_after()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                self.next_char();
                is_int = false;
            }
        }

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }

        let end = self.current;

        let number_str = self.src[start..end].iter().collect::<String>();

        Ok(if is_int {
            Token::Literal(Literal::Integer(number_str.parse().map_err(|_e| {
                LoxError::ScannerError(ErrorData {
                    message: format!("Could not parse integer from {}", number_str),
                    line_no: self.line_no,
                    location: String::new(),
                })
            })?))
        } else {
            Token::Literal(Literal::Number(number_str.parse().map_err(|_e| {
                LoxError::ScannerError(ErrorData {
                    message: format!("Could not parse number from {}", number_str),
                    line_no: self.line_no,
                    location: String::new(),
                })
            })?))
        })
    }

    fn match_identifier(&mut self) -> Result<Token, LoxError> {
        let start = self.current - 1;
        while let Some(c) = self.peek_char() {
            if is_identifier_char(c) {
                self.next_char();
            } else {
                break;
            }
        }
        let word = self.src[start..self.current].iter().collect::<String>();
        if let Some(token) = RESERVED_WORDS.get(&word[..]) {
            Ok(token.clone())
        } else {
            Ok(Token::Literal(Literal::Identifier(word)))
        }
    }
}

fn can_start_identifier(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_identifier_char(c: char) -> bool {
    can_start_identifier(c) || c.is_ascii_digit()
}
