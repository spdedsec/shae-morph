#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Output(OutputDefinition),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputDefinition {
    pub name: String,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralValue),
    Path(Vec<String>),
    Object(Vec<(String, Expr)>),
    Array(Vec<Expr>),
    Fallback {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    ArrayMap {
        source_path: Vec<String>,
        select_expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
