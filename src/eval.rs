use crate::ast::{Expr, LiteralValue, OutputDefinition, Program, Statement};
use serde_json::{Map, Number, Value};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum EvalError {
    #[error("Evaluation error: {0}")]
    General(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShaeValue {
    Json(Value),
    Empty,
}

impl ShaeValue {
    pub fn is_empty(&self) -> bool {
        match self {
            ShaeValue::Empty => true,
            ShaeValue::Json(Value::Null) => true,
            _ => false,
        }
    }

    pub fn into_json(self) -> Value {
        match self {
            ShaeValue::Json(v) => v,
            ShaeValue::Empty => Value::Null,
        }
    }
}

pub struct Evaluator<'a> {
    root: &'a Value,
}

impl<'a> Evaluator<'a> {
    pub fn new(root: &'a Value) -> Self {
        Self { root }
    }

    pub fn eval_program(&self, program: &Program) -> Result<Value, EvalError> {
        if program.statements.is_empty() {
            return Ok(Value::Object(Map::new()));
        }

        // If there's only one output definition, yield its direct result
        if program.statements.len() == 1 {
            match &program.statements[0] {
                Statement::Output(out) => {
                    let result = self.eval_expr(&out.expr, self.root)?;
                    return Ok(result.into_json());
                }
            }
        }

        // Multiple outputs: map output names to results
        let mut root_map = Map::new();
        for stmt in &program.statements {
            match stmt {
                Statement::Output(OutputDefinition { name, expr }) => {
                    let val = self.eval_expr(expr, self.root)?;
                    root_map.insert(name.clone(), val.into_json());
                }
            }
        }

        Ok(Value::Object(root_map))
    }

    pub fn eval_expr(&self, expr: &Expr, context: &Value) -> Result<ShaeValue, EvalError> {
        match expr {
            Expr::Literal(lit) => Ok(ShaeValue::Json(match lit {
                LiteralValue::String(s) => Value::String(s.clone()),
                LiteralValue::Number(n) => {
                    if let Some(num) = Number::from_f64(*n) {
                        Value::Number(num)
                    } else {
                        Value::Null
                    }
                }
                LiteralValue::Bool(b) => Value::Bool(*b),
                LiteralValue::Null => Value::Null,
            })),

            Expr::Path(segments) => {
                let mut current = context;
                for segment in segments {
                    match current {
                        Value::Object(map) => {
                            if let Some(next) = map.get(segment) {
                                current = next;
                            } else {
                                return Ok(ShaeValue::Empty);
                            }
                        }
                        _ => return Ok(ShaeValue::Empty),
                    }
                }

                if current.is_null() {
                    Ok(ShaeValue::Empty)
                } else {
                    Ok(ShaeValue::Json(current.clone()))
                }
            }

            Expr::Fallback { left, right } => {
                let left_val = self.eval_expr(left, context)?;
                if left_val.is_empty() {
                    self.eval_expr(right, context)
                } else {
                    Ok(left_val)
                }
            }

            Expr::Object(fields) => {
                let mut map = Map::new();
                for (key, val_expr) in fields {
                    let val = self.eval_expr(val_expr, context)?;
                    match val {
                        ShaeValue::Json(v) => {
                            map.insert(key.clone(), v);
                        }
                        ShaeValue::Empty => {
                            // If a field evaluates to Empty without fallback, omit the key
                        }
                    }
                }
                Ok(ShaeValue::Json(Value::Object(map)))
            }

            Expr::Array(items) => {
                let mut list = Vec::new();
                for item in items {
                    let val = self.eval_expr(item, context)?;
                    list.push(val.into_json());
                }
                Ok(ShaeValue::Json(Value::Array(list)))
            }

            Expr::ArrayMap {
                source_path,
                select_expr,
            } => {
                let mut current = context;
                for segment in source_path {
                    match current {
                        Value::Object(map) => {
                            if let Some(next) = map.get(segment) {
                                current = next;
                            } else {
                                return Ok(ShaeValue::Json(Value::Array(Vec::new())));
                            }
                        }
                        _ => return Ok(ShaeValue::Json(Value::Array(Vec::new()))),
                    }
                }

                match current {
                    Value::Array(items) => {
                        let mut result = Vec::new();
                        for item in items {
                            let mapped = self.eval_expr(select_expr, item)?;
                            result.push(mapped.into_json());
                        }
                        Ok(ShaeValue::Json(Value::Array(result)))
                    }
                    _ => Ok(ShaeValue::Json(Value::Array(Vec::new()))),
                }
            }
        }
    }
}

pub fn evaluate(program: &Program, input: &Value) -> Result<Value, EvalError> {
    Evaluator::new(input).eval_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use serde_json::json;

    #[test]
    fn test_eval_basic_mapping() {
        let code = r#"
            out Profile = {
                first_name: .user.name,
                age: .user.meta.age ?? 20,
                active: true
            }
        "#;

        let input = json!({
            "user": {
                "name": "Satya"
            }
        });

        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();
        let output = evaluate(&program, &input).unwrap();

        assert_eq!(
            output,
            json!({
                "first_name": "Satya",
                "age": 20.0,
                "active": true
            })
        );
    }

    #[test]
    fn test_eval_nested_fallback() {
        let code = r#"
            out Result = {
                val: .missing1 ?? .missing2 ?? "default"
            }
        "#;
        let input = json!({});
        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();
        let output = evaluate(&program, &input).unwrap();

        assert_eq!(output, json!({ "val": "default" }));
    }

    #[test]
    fn test_eval_implicit_array_map() {
        let code = r#"
            out DatabaseRows = from .customers[] select {
                user_id: .id,
                full_name: .meta.name ?? "Unknown User"
            }
        "#;

        let input = json!({
            "customers": [
                { "id": "cus_1", "meta": { "name": "Satya" } },
                { "id": "cus_2" }
            ]
        });

        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();
        let output = evaluate(&program, &input).unwrap();

        assert_eq!(
            output,
            json!([
                { "user_id": "cus_1", "full_name": "Satya" },
                { "user_id": "cus_2", "full_name": "Unknown User" }
            ])
        );
    }
}
