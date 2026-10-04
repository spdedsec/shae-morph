use crate::token::{SpannedToken, Token};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum LexerError {
    #[error("Unexpected character '{0}' at line {1}, col {2}")]
    UnexpectedChar(char, usize, usize),

    #[error("Unterminated string literal at line {0}, col {1}")]
    UnterminatedString(usize, usize),

    #[error("Invalid number format at line {0}, col {1}: {2}")]
    InvalidNumber(usize, usize, String),

    #[error("Unknown escape sequence '\\{0}' at line {1}, col {2}")]
    InvalidEscape(char, usize, usize),
}

pub struct Lexer {
    chars: Vec<(usize, char)>, // (byte_offset, char)
    cursor: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.char_indices().collect(),
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<SpannedToken>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(&(_, ch)) = self.current() {
            match ch {
                // Whitespace
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.cursor += 1;
                    self.line += 1;
                    self.col = 1;
                }

                // Single-line Comments
                '/' if self.peek(1) == Some('/') => {
                    self.advance(); // skip first '/'
                    self.advance(); // skip second '/'
                    while let Some(&(_, next_ch)) = self.current() {
                        if next_ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }

                // Symbols
                '.' => {
                    let start_col = self.col;
                    self.advance();
                    tokens.push(SpannedToken {
                        token: Token::Dot,
                        line: self.line,
                        col: start_col,
                    });
                }
                ':' => {
                    let start_col = self.col;
                    self.advance();
                    tokens.push(SpannedToken {
                        token: Token::Colon,
                        line: self.line,
                        col: start_col,
                    });
                }
                ',' => {
                    let start_col = self.col;
                    self.advance();
                    tokens.push(SpannedToken {
                        token: Token::Comma,
                        line: self.line,
                        col: start_col,
                    });
                }
                '=' => {
                    let start_col = self.col;
                    self.advance();
                    tokens.push(SpannedToken {
                        token: Token::Equals,
                        line: self.line,
                        col: start_col,
                    });
                }
                '{' => {
                    let start_col = self.col;
                    self.advance();
                    tokens.push(SpannedToken {
                        token: Token::LBrace,
                        line: self.line,
                        col: start_col,
                    });
                }
                '}' => {
                    let start_col = self.col;
                    self.advance();
                    tokens.push(SpannedToken {
                        token: Token::RBrace,
                        line: self.line,
                        col: start_col,
                    });
                }
                '[' => {
                    let start_col = self.col;
                    self.advance();
                    tokens.push(SpannedToken {
                        token: Token::LBracket,
                        line: self.line,
                        col: start_col,
                    });
                }
                ']' => {
                    let start_col = self.col;
                    self.advance();
                    tokens.push(SpannedToken {
                        token: Token::RBracket,
                        line: self.line,
                        col: start_col,
                    });
                }

                // Double question mark (??)
                '?' if self.peek(1) == Some('?') => {
                    let start_col = self.col;
                    self.advance(); // first '?'
                    self.advance(); // second '?'
                    tokens.push(SpannedToken {
                        token: Token::DoubleQuestion,
                        line: self.line,
                        col: start_col,
                    });
                }

                // Strings
                '"' => {
                    let token = self.scan_string()?;
                    tokens.push(token);
                }

                // Numbers
                '0'..='9' => {
                    let token = self.scan_number()?;
                    tokens.push(token);
                }

                // Identifiers or Keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    let token = self.scan_identifier();
                    tokens.push(token);
                }

                _ => {
                    return Err(LexerError::UnexpectedChar(ch, self.line, self.col));
                }
            }
        }

        tokens.push(SpannedToken {
            token: Token::Eof,
            line: self.line,
            col: self.col,
        });

        Ok(tokens)
    }

    fn current(&self) -> Option<&(usize, char)> {
        self.chars.get(self.cursor)
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.cursor + offset).map(|&(_, ch)| ch)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(&(_, ch)) = self.current() {
            self.cursor += 1;
            self.col += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn scan_string(&mut self) -> Result<SpannedToken, LexerError> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance(); // Consume opening quote

        let mut val = String::new();

        while let Some(&(_, ch)) = self.current() {
            match ch {
                '"' => {
                    self.advance(); // Consume closing quote
                    return Ok(SpannedToken {
                        token: Token::StringLit(val),
                        line: start_line,
                        col: start_col,
                    });
                }
                '\\' => {
                    self.advance();
                    if let Some(&(_, esc)) = self.current() {
                        self.advance();
                        match esc {
                            'n' => val.push('\n'),
                            't' => val.push('\t'),
                            'r' => val.push('\r'),
                            '"' => val.push('"'),
                            '\\' => val.push('\\'),
                            other => {
                                return Err(LexerError::InvalidEscape(other, self.line, self.col));
                            }
                        }
                    } else {
                        return Err(LexerError::UnterminatedString(start_line, start_col));
                    }
                }
                '\n' => {
                    return Err(LexerError::UnterminatedString(start_line, start_col));
                }
                other => {
                    val.push(other);
                    self.advance();
                }
            }
        }

        Err(LexerError::UnterminatedString(start_line, start_col))
    }

    fn scan_number(&mut self) -> Result<SpannedToken, LexerError> {
        let start_line = self.line;
        let start_col = self.col;
        let mut s = String::new();
        let mut seen_dot = false;

        while let Some(&(_, ch)) = self.current() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.advance();
            } else if ch == '.' && !seen_dot {
                // Peek ahead: if followed by digit, it's a floating point number.
                // Otherwise it's a dot operator (like in a path or chaining).
                if let Some(next_ch) = self.peek(1) {
                    if next_ch.is_ascii_digit() {
                        seen_dot = true;
                        s.push('.');
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let num = s
            .parse::<f64>()
            .map_err(|e| LexerError::InvalidNumber(start_line, start_col, e.to_string()))?;

        Ok(SpannedToken {
            token: Token::NumberLit(num),
            line: start_line,
            col: start_col,
        })
    }

    fn scan_identifier(&mut self) -> SpannedToken {
        let start_line = self.line;
        let start_col = self.col;
        let mut s = String::new();

        while let Some(&(_, ch)) = self.current() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let token = match s.as_str() {
            "out" => Token::Out,
            "in" => Token::In,
            "shape" => Token::Shape,
            "from" => Token::From,
            "select" => Token::Select,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            _ => Token::Ident(s),
        };

        SpannedToken {
            token,
            line: start_line,
            col: start_col,
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, LexerError> {
    Lexer::new(input).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lexing() {
        let code = r#"
            out Profile = {
                first_name: .user.name,
                age: .user.meta.age ?? 20,
                active: true
            }
        "#;

        let tokens = tokenize(code).expect("Tokenization failed");
        let kinds: Vec<Token> = tokens.into_iter().map(|st| st.token).collect();

        assert_eq!(
            kinds,
            vec![
                Token::Out,
                Token::Ident("Profile".to_string()),
                Token::Equals,
                Token::LBrace,
                Token::Ident("first_name".to_string()),
                Token::Colon,
                Token::Dot,
                Token::Ident("user".to_string()),
                Token::Dot,
                Token::Ident("name".to_string()),
                Token::Comma,
                Token::Ident("age".to_string()),
                Token::Colon,
                Token::Dot,
                Token::Ident("user".to_string()),
                Token::Dot,
                Token::Ident("meta".to_string()),
                Token::Dot,
                Token::Ident("age".to_string()),
                Token::DoubleQuestion,
                Token::NumberLit(20.0),
                Token::Comma,
                Token::Ident("active".to_string()),
                Token::Colon,
                Token::True,
                Token::RBrace,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_comments_and_strings() {
        let code = r#"
            // This is a comment
            out Test = {
                greeting: "Hello, \"World\"!\n"
            }
        "#;
        let tokens = tokenize(code).unwrap();
        assert_eq!(tokens[0].token, Token::Out);
        assert_eq!(tokens[1].token, Token::Ident("Test".to_string()));
        assert_eq!(tokens[2].token, Token::Equals);
        assert_eq!(tokens[3].token, Token::LBrace);
        assert_eq!(tokens[4].token, Token::Ident("greeting".to_string()));
        assert_eq!(tokens[5].token, Token::Colon);
        assert_eq!(
            tokens[6].token,
            Token::StringLit("Hello, \"World\"!\n".to_string())
        );
        assert_eq!(tokens[7].token, Token::RBrace);
    }
}
