use crate::ast::{BinaryOp, Expr, LiteralValue, OutputDefinition, Program, Statement, UnaryOp};
use crate::token::{SpannedToken, Token};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("Shae Syntax Hiccup: Unexpected {0} at line {1}, col {2}. {3}")]
    UnexpectedToken(String, usize, usize, String),

    #[error("Shae Syntax Hiccup: Unexpected end of file while parsing {0}. Did you leave an expression hanging?")]
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
                "Expected statement keyword ('out').".to_string(),
            )),
        }
    }

    fn parse_output_definition(&mut self) -> Result<Statement, ParserError> {
        self.consume(&Token::Out, "Expected 'out' to start an output declaration.")?;

        let name = match self.advance().token {
            Token::Ident(name) => name,
            other => {
                let prev = self.previous();
                return Err(ParserError::UnexpectedToken(
                    other.to_string(),
                    prev.line,
                    prev.col,
                    "Expected an output name (e.g., 'out Profile = { ... }').".to_string(),
                ));
            }
        };

        self.consume(&Token::Equals, "Expected '=' after the output name.")?;

        let expr = self.parse_expression(0)?;

        Ok(Statement::Output(OutputDefinition { name, expr }))
    }

    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParserError> {
        let mut left = self.parse_prefix()?;

        // Postfix & Infix operator climbing
        loop {
            // Check for postfix method call: .method(...)
            if self.is_method_call_ahead() {
                self.advance(); // consume '.'
                let method_name = match self.advance().token {
                    Token::Ident(name) => name,
                    other => {
                        let prev = self.previous();
                        return Err(ParserError::UnexpectedToken(
                            other.to_string(),
                            prev.line,
                            prev.col,
                            "Expected method name after '.'".to_string(),
                        ));
                    }
                };

                self.consume(&Token::LParen, "Expected '(' after method name")?;
                let mut args = Vec::new();
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    let arg = self.parse_expression(0)?;
                    args.push(arg);
                    if self.match_token(&Token::Comma) {
                        continue;
                    } else if !self.check(&Token::RParen) {
                        break;
                    }
                }
                self.consume(&Token::RParen, "Expected ')' to close method call arguments")?;

                left = Expr::MethodCall {
                    target: Box::new(left),
                    method: method_name,
                    args,
                };
                continue;
            }

            let current = self.peek();
            if let Some((l_bp, r_bp, op)) = self.infix_binding_power(&current.token) {
                if l_bp < min_bp {
                    break;
                }

                self.advance(); // consume operator
                match op {
                    InfixOp::Fallback => {
                        let right = self.parse_expression(r_bp)?;
                        left = Expr::Fallback {
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                    }
                    InfixOp::Binary(binary_op) => {
                        let right = self.parse_expression(r_bp)?;
                        left = Expr::BinaryOp {
                            left: Box::new(left),
                            op: binary_op,
                            right: Box::new(right),
                        };
                    }
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn is_method_call_ahead(&self) -> bool {
        if self.check(&Token::Dot) {
            if let Some(t1) = self.peek_offset(1) {
                if matches!(t1.token, Token::Ident(_)) {
                    if let Some(t2) = self.peek_offset(2) {
                        return matches!(t2.token, Token::LParen);
                    }
                }
            }
        }
        false
    }

    fn infix_binding_power(&self, token: &Token) -> Option<(u8, u8, InfixOp)> {
        match token {
            Token::Or => Some((1, 2, InfixOp::Binary(BinaryOp::Or))),
            Token::And => Some((3, 4, InfixOp::Binary(BinaryOp::And))),
            Token::EqEq => Some((5, 6, InfixOp::Binary(BinaryOp::Eq))),
            Token::NotEq => Some((5, 6, InfixOp::Binary(BinaryOp::NotEq))),
            Token::Lt => Some((7, 8, InfixOp::Binary(BinaryOp::Lt))),
            Token::LtEq => Some((7, 8, InfixOp::Binary(BinaryOp::LtEq))),
            Token::Gt => Some((7, 8, InfixOp::Binary(BinaryOp::Gt))),
            Token::GtEq => Some((7, 8, InfixOp::Binary(BinaryOp::GtEq))),
            Token::DoubleQuestion => Some((9, 8, InfixOp::Fallback)),
            Token::Plus => Some((11, 12, InfixOp::Binary(BinaryOp::Add))),
            _ => None,
        }
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParserError> {
        let current = self.peek();
        match &current.token {
            Token::Not | Token::Bang => {
                self.advance();
                let expr = self.parse_expression(14)?;
                Ok(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression(0)?;
                self.consume(&Token::RParen, "Expected ')' to close grouped expression.")?;
                Ok(expr)
            }
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
                "Expected an expression (path '.', literal, '{', '[', or 'from').".to_string(),
            )),
        }
    }

    fn parse_path(&mut self) -> Result<Expr, ParserError> {
        let mut segments = Vec::new();

        self.consume(&Token::Dot, "Expected '.' to start path")?;

        let first_ident = match self.advance().token {
            Token::Ident(s) => s,
            other => {
                let prev = self.previous();
                return Err(ParserError::UnexpectedToken(
                    other.to_string(),
                    prev.line,
                    prev.col,
                    "Expected property identifier following '.' in path (e.g., .user.name)."
                        .to_string(),
                ));
            }
        };
        segments.push(first_ident);

        // Followed by zero or more `.ident`, UNLESS it's a method call `.method(...)`
        while self.check(&Token::Dot) {
            // Peek if next ident is followed by `(` (method call)
            if self.is_method_call_ahead() {
                break;
            }

            self.advance(); // consume '.'
            let next_ident = match self.advance().token {
                Token::Ident(s) => s,
                other => {
                    let prev = self.previous();
                    return Err(ParserError::UnexpectedToken(
                        other.to_string(),
                        prev.line,
                        prev.col,
                        "Expected property identifier following '.' in path.".to_string(),
                    ));
                }
            };
            segments.push(next_ident);
        }

        Ok(Expr::Path(segments))
    }

    fn parse_object(&mut self) -> Result<Expr, ParserError> {
        self.consume(&Token::LBrace, "Expected '{' to start an object")?;
        let mut fields = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let key = match self.advance().token {
                Token::Ident(name) => name,
                Token::StringLit(name) => name,
                other => {
                    let prev = self.previous();
                    return Err(ParserError::UnexpectedToken(
                        other.to_string(),
                        prev.line,
                        prev.col,
                        "Expected object key name (identifier or string).".to_string(),
                    ));
                }
            };

            self.consume(&Token::Colon, "Expected ':' after object key")?;
            let val = self.parse_expression(0)?;
            fields.push((key, val));

            if self.match_token(&Token::Comma) {
                continue;
            } else if !self.check(&Token::RBrace) {
                continue;
            }
        }

        self.consume(
            &Token::RBrace,
            "Expected '}' to close object. Did you forget to close the brace?",
        )?;
        Ok(Expr::Object(fields))
    }

    fn parse_array(&mut self) -> Result<Expr, ParserError> {
        self.consume(&Token::LBracket, "Expected '[' to start array")?;
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

        self.consume(&Token::RBracket, "Expected ']' to close array")?;
        Ok(Expr::Array(items))
    }

    fn parse_array_map(&mut self) -> Result<Expr, ParserError> {
        self.consume(&Token::From, "Expected 'from' to start array projection")?;

        let path_expr = self.parse_path()?;
        let source_path = match path_expr {
            Expr::Path(p) => p,
            _ => unreachable!(),
        };

        // If optional `[]` follows path, consume it
        if self.match_token(&Token::LBracket) {
            self.consume(
                &Token::RBracket,
                "Expected ']' after '[' in array source path",
            )?;
        }

        // Optional `where` filter clause
        let filter = if self.match_token(&Token::Where) {
            Some(Box::new(self.parse_expression(0)?))
        } else {
            None
        };

        self.consume(
            &Token::Select,
            "Expected 'select' after array source / where clause",
        )?;
        let select_expr = self.parse_expression(0)?;

        Ok(Expr::ArrayMap {
            source_path,
            filter,
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

    fn peek_offset(&self, offset: usize) -> Option<&SpannedToken> {
        self.tokens.get(self.cursor + offset)
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

enum InfixOp {
    Fallback,
    Binary(BinaryOp),
}

pub fn parse(tokens: Vec<SpannedToken>) -> Result<Program, ParserError> {
    Parser::new(tokens).parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_where_filter() {
        let code = r#"
            out Filtered = from .users[] where .age >= 18 and .is_active == true select {
                id: .id,
                name: .name.uppercase()
            }
        "#;
        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Output(out) => match &out.expr {
                Expr::ArrayMap { filter, .. } => {
                    assert!(filter.is_some());
                }
                _ => panic!("Expected ArrayMap"),
            },
        }
    }
}
