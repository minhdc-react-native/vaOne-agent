use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::pdf::template::formatter::FORMATTERS;
use crate::pdf::utils::resolve_value;

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(expr: &str, data: &Value) -> Result<String> {
        let expr = expr.trim();

        // function(...)
        if expr.ends_with(')') {
            return Self::eval_function(expr, data);
        }

        // variable
        Ok(resolve_value(data, expr).unwrap_or_default())
    }

    fn eval_function(expr: &str, data: &Value) -> Result<String> {
        let pos = expr
            .find('(')
            .ok_or_else(|| anyhow!("Invalid expression: {}", expr))?;

        let name = expr[..pos].trim();

        let args_str = &expr[pos + 1..expr.len() - 1];

        let args = Self::parse_arguments(args_str, data);

        FORMATTERS.call(name, &args)
    }

    fn parse_arguments(args: &str, data: &Value) -> Vec<Value> {
        let mut result = Vec::new();

        if args.trim().is_empty() {
            return result;
        }

        let mut current = String::new();
        let mut in_string = false;

        for ch in args.chars() {
            match ch {
                '"' => {
                    in_string = !in_string;
                    current.push(ch);
                }

                ',' if !in_string => {
                    result.push(Self::eval_argument(current.trim(), data));
                    current.clear();
                }

                _ => current.push(ch),
            }
        }

        if !current.trim().is_empty() {
            result.push(Self::eval_argument(current.trim(), data));
        }

        result
    }

    fn eval_argument(arg: &str, data: &Value) -> Value {
        let arg = arg.trim();

        // "abc"
        if arg.starts_with('"') && arg.ends_with('"') {
            return Value::String(arg[1..arg.len() - 1].to_string());
        }

        // number
        if let Ok(v) = arg.parse::<i64>() {
            return Value::Number(v.into());
        }

        if let Ok(v) = arg.parse::<f64>() {
            return serde_json::json!(v);
        }

        // variable
        if let Some(value) = resolve_value(data, arg) {
            return serde_json::json!(value);
        }

        Value::Null
    }
}
