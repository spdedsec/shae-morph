use crate::ast::{BinaryOp, Expr, LiteralValue, OutputDefinition, Program, Statement, UnaryOp};
use serde_json::{Map, Number, Value};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum EvalError {
    #[error("Shae Runtime Error: {0}")]
    General(String),

    #[error("Shae Runtime Error: Method '{method}' is unknown on {type_name}. {hint}")]
    UnknownMethod {
        method: String,
        type_name: String,
        hint: String,
    },
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

    pub fn is_truthy(&self) -> bool {
        match self {
            ShaeValue::Empty => false,
            ShaeValue::Json(Value::Null) => false,
            ShaeValue::Json(Value::Bool(b)) => *b,
            ShaeValue::Json(Value::Number(n)) => n.as_f64().map_or(false, |f| f != 0.0),
            ShaeValue::Json(Value::String(s)) => !s.is_empty(),
            ShaeValue::Json(Value::Array(arr)) => !arr.is_empty(),
            ShaeValue::Json(Value::Object(obj)) => !obj.is_empty(),
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

        if program.statements.len() == 1 {
            match &program.statements[0] {
                Statement::Output(out) => {
                    let result = self.eval_expr(&out.expr, self.root)?;
                    return Ok(result.into_json());
                }
            }
        }

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

            Expr::UnaryOp { op, expr } => {
                let val = self.eval_expr(expr, context)?;
                match op {
                    UnaryOp::Not => Ok(ShaeValue::Json(Value::Bool(!val.is_truthy()))),
                }
            }

            Expr::BinaryOp { left, op, right } => {
                let left_val = self.eval_expr(left, context)?;
                let right_val = self.eval_expr(right, context)?;

                match op {
                    BinaryOp::And => {
                        if !left_val.is_truthy() {
                            Ok(left_val)
                        } else {
                            Ok(right_val)
                        }
                    }
                    BinaryOp::Or => {
                        if left_val.is_truthy() {
                            Ok(left_val)
                        } else {
                            Ok(right_val)
                        }
                    }
                    BinaryOp::Eq => {
                        let l_json = left_val.into_json();
                        let r_json = right_val.into_json();
                        Ok(ShaeValue::Json(Value::Bool(l_json == r_json)))
                    }
                    BinaryOp::NotEq => {
                        let l_json = left_val.into_json();
                        let r_json = right_val.into_json();
                        Ok(ShaeValue::Json(Value::Bool(l_json != r_json)))
                    }
                    BinaryOp::Add => {
                        match (left_val, right_val) {
                            (
                                ShaeValue::Json(Value::Number(n1)),
                                ShaeValue::Json(Value::Number(n2)),
                            ) => {
                                let sum = n1.as_f64().unwrap_or(0.0) + n2.as_f64().unwrap_or(0.0);
                                Ok(ShaeValue::Json(json_number(sum)))
                            }
                            (ShaeValue::Json(v1), ShaeValue::Json(v2)) => {
                                // String concatenation
                                let s1 = value_to_string(&v1);
                                let s2 = value_to_string(&v2);
                                Ok(ShaeValue::Json(Value::String(format!("{}{}", s1, s2))))
                            }
                            _ => Ok(ShaeValue::Empty),
                        }
                    }
                    BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => {
                        let l_json = left_val.into_json();
                        let r_json = right_val.into_json();
                        let cmp = match (l_json, r_json) {
                            (Value::Number(n1), Value::Number(n2)) => {
                                let f1 = n1.as_f64().unwrap_or(0.0);
                                let f2 = n2.as_f64().unwrap_or(0.0);
                                f1.partial_cmp(&f2)
                            }
                            (Value::String(s1), Value::String(s2)) => Some(s1.cmp(&s2)),
                            _ => None,
                        };

                        let res = match cmp {
                            Some(std::cmp::Ordering::Less) => {
                                matches!(op, BinaryOp::Lt | BinaryOp::LtEq)
                            }
                            Some(std::cmp::Ordering::Equal) => {
                                matches!(op, BinaryOp::LtEq | BinaryOp::GtEq)
                            }
                            Some(std::cmp::Ordering::Greater) => {
                                matches!(op, BinaryOp::Gt | BinaryOp::GtEq)
                            }
                            None => false,
                        };
                        Ok(ShaeValue::Json(Value::Bool(res)))
                    }
                }
            }

            Expr::MethodCall {
                target,
                method,
                args,
            } => {
                let target_val = self.eval_expr(target, context)?;
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, context)?.into_json());
                }

                self.dispatch_method(target_val, method, &evaluated_args)
            }

            Expr::Object(fields) => {
                let mut map = Map::new();
                for (key, val_expr) in fields {
                    let val = self.eval_expr(val_expr, context)?;
                    match val {
                        ShaeValue::Json(v) => {
                            map.insert(key.clone(), v);
                        }
                        ShaeValue::Empty => {}
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
                filter,
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
                            // Check where filter condition if present
                            if let Some(filter_expr) = filter {
                                let filter_res = self.eval_expr(filter_expr, item)?;
                                if !filter_res.is_truthy() {
                                    continue;
                                }
                            }

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

    fn dispatch_method(
        &self,
        target: ShaeValue,
        method: &str,
        args: &[Value],
    ) -> Result<ShaeValue, EvalError> {
        if target.is_empty() {
            return Ok(ShaeValue::Empty);
        }

        let json_val = target.into_json();
        match json_val {
            Value::String(s) => match method {
                "uppercase" => Ok(ShaeValue::Json(Value::String(s.to_uppercase()))),
                "lowercase" => Ok(ShaeValue::Json(Value::String(s.to_lowercase()))),
                "trim" => Ok(ShaeValue::Json(Value::String(s.trim().to_string()))),
                "capitalize" => {
                    let mut chars = s.chars();
                    let capitalized = match chars.next() {
                        None => String::new(),
                        Some(first) => {
                            first.to_uppercase().collect::<String>() + chars.as_str()
                        }
                    };
                    Ok(ShaeValue::Json(Value::String(capitalized)))
                }
                "len" => Ok(ShaeValue::Json(json_number(s.chars().count() as f64))),
                "toInt" => {
                    if let Ok(num) = s.trim().parse::<i64>() {
                        Ok(ShaeValue::Json(Value::Number(num.into())))
                    } else if let Ok(num) = s.trim().parse::<f64>() {
                        Ok(ShaeValue::Json(json_number(num.floor())))
                    } else {
                        Ok(ShaeValue::Empty)
                    }
                }
                "toNumber" | "toFloat" => {
                    if let Ok(num) = s.trim().parse::<f64>() {
                        Ok(ShaeValue::Json(json_number(num)))
                    } else {
                        Ok(ShaeValue::Empty)
                    }
                }
                "contains" => {
                    let sub = args.get(0).and_then(|v| v.as_str()).unwrap_or("");
                    Ok(ShaeValue::Json(Value::Bool(s.contains(sub))))
                }
                "startsWith" => {
                    let sub = args.get(0).and_then(|v| v.as_str()).unwrap_or("");
                    Ok(ShaeValue::Json(Value::Bool(s.starts_with(sub))))
                }
                "endsWith" => {
                    let sub = args.get(0).and_then(|v| v.as_str()).unwrap_or("");
                    Ok(ShaeValue::Json(Value::Bool(s.ends_with(sub))))
                }
                "split" => {
                    let delimiter = args.get(0).and_then(|v| v.as_str()).unwrap_or(",");
                    let parts: Vec<Value> = s
                        .split(delimiter)
                        .map(|p| Value::String(p.to_string()))
                        .collect();
                    Ok(ShaeValue::Json(Value::Array(parts)))
                }
                "replace" => {
                    let from = args.get(0).and_then(|v| v.as_str()).unwrap_or("");
                    let to = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
                    Ok(ShaeValue::Json(Value::String(s.replace(from, to))))
                }
                other => Err(EvalError::UnknownMethod {
                    method: other.to_string(),
                    type_name: "String".to_string(),
                    hint: "Available: .uppercase(), .lowercase(), .trim(), .capitalize(), .len(), .toInt(), .toFloat(), .split(sep), .replace(from, to), .contains(sub)".to_string(),
                }),
            },

            Value::Number(n) => match method {
                "toString" => Ok(ShaeValue::Json(Value::String(n.to_string()))),
                "toInt" => {
                    let val = n.as_f64().unwrap_or(0.0).floor();
                    Ok(ShaeValue::Json(json_number(val)))
                }
                "round" => {
                    let val = n.as_f64().unwrap_or(0.0).round();
                    Ok(ShaeValue::Json(json_number(val)))
                }
                "floor" => {
                    let val = n.as_f64().unwrap_or(0.0).floor();
                    Ok(ShaeValue::Json(json_number(val)))
                }
                "ceil" => {
                    let val = n.as_f64().unwrap_or(0.0).ceil();
                    Ok(ShaeValue::Json(json_number(val)))
                }
                "abs" => {
                    let val = n.as_f64().unwrap_or(0.0).abs();
                    Ok(ShaeValue::Json(json_number(val)))
                }
                other => Err(EvalError::UnknownMethod {
                    method: other.to_string(),
                    type_name: "Number".to_string(),
                    hint: "Available: .toString(), .toInt(), .round(), .floor(), .ceil(), .abs()".to_string(),
                }),
            },

            Value::Array(arr) => match method {
                "len" => Ok(ShaeValue::Json(json_number(arr.len() as f64))),
                "first" => match arr.first() {
                    Some(first) => Ok(ShaeValue::Json(first.clone())),
                    None => Ok(ShaeValue::Empty),
                },
                "last" => match arr.last() {
                    Some(last) => Ok(ShaeValue::Json(last.clone())),
                    None => Ok(ShaeValue::Empty),
                },
                "reverse" => {
                    let mut reversed = arr.clone();
                    reversed.reverse();
                    Ok(ShaeValue::Json(Value::Array(reversed)))
                }
                "join" => {
                    let sep = args.get(0).and_then(|v| v.as_str()).unwrap_or(",");
                    let joined: Vec<String> = arr.iter().map(value_to_string).collect();
                    Ok(ShaeValue::Json(Value::String(joined.join(sep))))
                }
                other => Err(EvalError::UnknownMethod {
                    method: other.to_string(),
                    type_name: "Array".to_string(),
                    hint: "Available: .len(), .first(), .last(), .reverse(), .join(delimiter)".to_string(),
                }),
            },

            Value::Object(obj) => match method {
                "keys" => {
                    let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
                    Ok(ShaeValue::Json(Value::Array(keys)))
                }
                "values" => {
                    let vals: Vec<Value> = obj.values().cloned().collect();
                    Ok(ShaeValue::Json(Value::Array(vals)))
                }
                "len" => Ok(ShaeValue::Json(json_number(obj.len() as f64))),
                other => Err(EvalError::UnknownMethod {
                    method: other.to_string(),
                    type_name: "Object".to_string(),
                    hint: "Available: .keys(), .values(), .len()".to_string(),
                }),
            },

            Value::Bool(b) => match method {
                "toString" => Ok(ShaeValue::Json(Value::String(b.to_string()))),
                other => Err(EvalError::UnknownMethod {
                    method: other.to_string(),
                    type_name: "Bool".to_string(),
                    hint: "Available: .toString()".to_string(),
                }),
            },

            Value::Null => Ok(ShaeValue::Empty),
        }
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(v).unwrap_or_default(),
    }
}

fn json_number(f: f64) -> Value {
    if let Some(n) = Number::from_f64(f) {
        Value::Number(n)
    } else {
        Value::Null
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
    fn test_eval_methods_and_transforms() {
        let code = r#"
            out Result = {
                title: .book.title.trim().uppercase(),
                pages: .book.pages_str.toInt() ?? 0,
                tags: .book.tags.join(" | ")
            }
        "#;

        let input = json!({
            "book": {
                "title": "  the hitchhiker's guide  ",
                "pages_str": "42",
                "tags": ["sci-fi", "comedy", "classic"]
            }
        });

        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();
        let output = evaluate(&program, &input).unwrap();

        assert_eq!(
            output,
            json!({
                "title": "THE HITCHHIKER'S GUIDE",
                "pages": 42,
                "tags": "sci-fi | comedy | classic"
            })
        );
    }

    #[test]
    fn test_eval_where_filter() {
        let code = r#"
            out Adults = from .people[] where .age >= 18 select {
                name: .name,
                is_senior: .age >= 60
            }
        "#;

        let input = json!({
            "people": [
                { "name": "Timmy", "age": 10 },
                { "name": "Satya", "age": 21 },
                { "name": "Gandalf", "age": 200 }
            ]
        });

        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();
        let output = evaluate(&program, &input).unwrap();

        assert_eq!(
            output,
            json!([
                { "name": "Satya", "is_senior": false },
                { "name": "Gandalf", "is_senior": true }
            ])
        );
    }

    #[test]
    fn test_eval_string_concatenation() {
        let code = r#"
            out Greet = {
                message: "Hello, " + .user.name + "!"
            }
        "#;
        let input = json!({ "user": { "name": "Satya" } });
        let tokens = tokenize(code).unwrap();
        let program = parse(tokens).unwrap();
        let output = evaluate(&program, &input).unwrap();

        assert_eq!(output, json!({ "message": "Hello, Satya!" }));
    }
}
