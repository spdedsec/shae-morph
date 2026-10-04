pub mod ast;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod token;

use eval::EvalError;
use lexer::LexerError;
use parser::ParserError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaeError {
    #[error("Lexer error: {0}")]
    Lexer(#[from] LexerError),

    #[error("Parser error: {0}")]
    Parser(#[from] ParserError),

    #[error("Runtime error: {0}")]
    Eval(#[from] EvalError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Executes a Shae script against a JSON input value and returns the transformed JSON value.
pub fn transform(source: &str, input: &serde_json::Value) -> Result<serde_json::Value, ShaeError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    let output = eval::evaluate(&program, input)?;
    Ok(output)
}
