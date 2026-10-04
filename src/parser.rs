use crate::ast::{Expr, LiteralValue, OutputDefinition, Program, Statement};
use crate::token::{SpannedToken, Token};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("Unexpected token {0} at line {1}, col {2}. Expected {3}")]
    UnexpectedToken(String, usize, usize, String),

    #[error("Unexpected end of file while parsing {0}")]
    UnexpectedEof(String),
}

pub struct Parser {
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        let current = self.peek();
        match &current.token {
            Token::Out => self.parse_output_definition(),
            other => Err(ParserError::UnexpectedToken(
                other.to_string(),
                current.line,
                current.col,
                "statement keyword ('out')".to_string(),
            )),
        }
    }

    fn parse_output_definition(&mut self) -> Result<Statement, ParserError> {
        self.consume(&Token::Out, "expected 'out'")?;

        let name = match self.advance().token {
            Token::Ident(name) => name,
            other => {
                let prev = self.previous();
                return Err(ParserError::UnexpectedToken(
                    other.to_string(),
                    prev.line,
                    prev.col,
                    "output identifier".to_string(),
                ));
            }
        };

        self.consume(&Token::Equals, "expected '=' after output name")?;

        let expr = self.parse_expression(0)?;

        Ok(Statement::Output(OutputDefinition { name, expr }))
    }

    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParserError> {
        let mut left = self.parse_primary()?;

        loop {
            let current = self.peek();
            if let Some((l_bp, r_bp)) = self.infix_binding_power(&current.token) {
                if l_bp < min_bp {
                    break;
                }

                let op_token = self.advance().token;
                match op_token {
                    Token::DoubleQuestion => {
                        let right = self.parse_expression(r_bp)?;
                        left = Expr::Fallback {
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    }
                    _ => unreachable!(),
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn infix_binding_power(&self, token: &Token) -> Option<(u8, u8)> {
        match token {
            // ?? is right-associative (or chained left-to-right)
            Token::DoubleQuestion => Some((10, 9)),
            _ => None,
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        let current = self.peek();
        match &current.token {
            Token::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(LiteralValue::String(s)))
            }
            Token::NumberLit(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(LiteralValue::Number(n)))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Bool(false)))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Literal(LiteralValue::Null))
            }
            Token::Dot => self.parse_path(),
            Token::LBrace => self.parse_object(),
            Token::LBracket => self.parse_array(),
            Token::From => self.parse_array_map(),
            other => Err(ParserError::UnexpectedToken(
                other.to_string(),
                current.line,
                current.col,
                "an expression (literal, path '.', '{', '[', or 'from')".to_string(),
            )),
        }
    }

    fn parse_path(&mut self) -> Result<Expr, ParserError> {
        let mut segments = Vec::new();

        // Must start with '.'
        self.consume(&Token::Dot, "expected '.' to start path")?;

        let first_ident = match self.advance().token {
            Token::Ident(s) => s,
            other => {
                let prev = self.previous();
                return Err(ParserError::UnexpectedToken(
                    other.to_string(),
                    prev.line,
                    prev.col,
                    "identifier following '.' in path".to_string(),
                ));
            }
        };
        segments.push(first_ident);

        // Followed by zero or more `.ident`
        while self.match_token(&Token::Dot) {
            let next_ident = match self.advance().token {
                Token::Ident(s) => s,
                other => {
                    let prev = self.previous();
                    return Err(ParserError::UnexpectedToken(
                        other.to_string(),
                        prev.line,
                        prev.col,
                        "identifier following '.' in path".to_string(),
                    ));
                }
            };
            segments.push(next_ident);
        }

        Ok(Expr::Path(segments))
    }

    fn parse_object(&mut self) -> Result<Expr, ParserError> {
        self.consume(&Token::LBrace, "expected '{'")?;
        let mut fields = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Key can be an Ident or a StringLit
            let key = match self.advance().token {
                Token::Ident(name) => name,
                Token::StringLit(name) => name,
                other => {
                    let prev = self.previous();
                    return Err(ParserError::UnexpectedToken(
                        other.to_string(),
                        prev.line,
                        prev.col,
                        "object key (identifier or string)".to_string(),
                    ));
                }
            };

            self.consume(&Token::Colon, "expected ':' after object key")?;
            let val = self.parse_expression(0)?;
            fields.push((key, val));

            // Optional comma between fields
            if self.match_token(&Token::Comma) {
                // If trailing comma before '}', it's allowed
                continue;
            } else if !self.check(&Token::RBrace) {
                // Allow newline-separated fields without comma
                continue;
            }
        }

        self.consume(&Token::RBrace, "expected '}' to close object")?;
        Ok(Expr::Object(fields))
    }

    fn parse_array(&mut self) -> Result<Expr, ParserError> {
        self.consume(&Token::LBracket, "expected '['")?;
        let mut items = Vec::new();

        while !self.check(&Token::RBracket) && !self.is_at_end() {
            let item = self.parse_expression(0)?;
            items.push(item);

            if self.match_token(&Token::Comma) {
                continue;
            } else if !self.check(&Token::RBracket) {
                continue;
            }
        }

        self.consume(&Token::RBracket, "expected ']' to close array")?;
        Ok(Expr::Array(items))
    }

    fn parse_array_map(&mut self) -> Result<Expr, ParserError> {
        self.consume(&Token::From, "expected 'from'")?;

        // Expect path like .users or .users[]
        let path_expr = self.parse_path()?;
        let source_path = match path_expr {
            Expr::Path(p) => p,
            _ => unreachable!(),
        };

        // If there's an optional `[]` after the path, consume it
        if self.match_token(&Token::LBracket) {
            self.consume(&Token::RBracket, "expected ']' after '[' in array source")?;
        }

        self.consume(&Token::Select, "expected 'select' after array source path")?;
        let select_expr = self.parse_expression(0)?;

        Ok(Expr::ArrayMap {
            source_path,
            select_expr: Box::new(select_expr),
        })
    }

    fn match_token(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            token == &Token::Eof
        } else {
            &self.peek().token == token
        }
    }

    fn advance(&mut self) -> SpannedToken {
        if !self.is_at_end() {
            self.cursor += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token == Token::Eof
    }

    fn peek(&self) -> SpannedToken {
        self.tokens
            .get(self.cursor)
            .cloned()
            .unwrap_or(SpannedToken {
                token: Token::Eof,
                line: 0,
                col: 0,
            })
    }

    fn previous(&self) -> SpannedToken {
        if self.cursor == 0 {
            self.peek()
        } else {
            self.tokens[self.cursor - 1].clone()
        }
    }

    fn consume(&mut self, expected: &Token, message: &str) -> Result<SpannedToken, ParserError> {
        if self.check(expected) {
            Ok(self.advance())
        } else {
            let current = self.peek();
            Err(ParserError::UnexpectedToken(
                current.token.to_string(),
                current.line,
                current.col,
                message.to_string(),
            ))
        }
    }
}

pub fn parse(tokens: Vec<SpannedToken>) -> Result<Program, ParserError> {
    Parser::new(tokens).parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_simple_output() {
        let code = r#"
            out Profile = {
                name: .user.name,
                age: .user.age ?? 18
            }
        "#;

        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Output(out) => {
                assert_eq!(out.name, "Profile");
                match &out.expr {
                    Expr::Object(fields) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].0, "name");
                        assert_eq!(fields[0].1, Expr::Path(vec!["user".into(), "name".into()]));
                        assert_eq!(fields[1].0, "age");
                        assert_eq!(
                            fields[1].1,
                            Expr::Fallback {
                                left: Box::new(Expr::Path(vec!["user".into(), "age".into()])),
                                right: Box::new(Expr::Literal(LiteralValue::Number(18.0))),
                            }
                        );
                    }
                    _ => panic!("Expected object expr"),
                }
            }
        }
    }

    #[test]
    fn test_parse_array_map() {
        let code = r#"
            out Customers = from .data.customers[] select {
                id: .id,
                email: .meta.email ?? "none"
            }
        "#;
        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Output(out) => {
                assert_eq!(out.name, "Customers");
                match &out.expr {
                    Expr::ArrayMap {
                        source_path,
                        select_expr,
                    } => {
                        assert_eq!(source_path, &vec!["data".to_string(), "customers".to_string()]);
                        match &**select_expr {
                            Expr::Object(fields) => {
                                assert_eq!(fields.len(), 2);
                                assert_eq!(fields[0].0, "id");
                                assert_eq!(fields[1].0, "email");
                            }
                            _ => panic!("Expected object inside select"),
                        }
                    }
                    _ => panic!("Expected ArrayMap expr"),
                }
            }
        }
    }
}
