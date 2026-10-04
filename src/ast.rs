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
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    ArrayMap {
        source_path: Vec<String>,
        filter: Option<Box<Expr>>,
        select_expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,    // +
    Eq,     // ==
    NotEq,  // !=
    Lt,     // <
    LtEq,   // <=
    Gt,     // >
    GtEq,   // >=
    And,    // and
    Or,     // or
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,    // not, !
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
