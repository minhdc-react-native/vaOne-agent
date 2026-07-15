use crate::template::{formatter::FORMATTERS, models::FormatterContext};
use crate::utils::{resolve_value, value_to_string};
use anyhow::{anyhow, Result};
use evalexpr::{
    eval_boolean_with_context, eval_number_with_context, ContextWithMutableFunctions,
    ContextWithMutableVariables, DefaultNumericTypes, Function, HashMapContext, Value as EvalValue,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize)]
struct VisibleIfConfig {
    #[serde(default)]
    watch: Vec<String>,

    visible: Option<String>,
}

use serde_json::Value;

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(expr: &str, data: &Value, ctx: FormatterContext) -> Result<String> {
        // function(...)
        if expr.ends_with(")") {
            return Self::eval_function(expr, data, ctx);
        }

        // variable
        // Ok(resolve_value(data, expr))
        Ok(resolve_value(data, expr)
            .map(|v| value_to_string(&v))
            .unwrap_or_else(|| format!("{{{}}}", expr)))
    }

    fn eval_function(expr: &str, data: &Value, ctx: FormatterContext) -> Result<String> {
        let pos = expr
            .find('(')
            .ok_or_else(|| anyhow!("Invalid expression: {}", expr))?;

        let name = expr[..pos].trim();

        let args_str = &expr[pos + 1..expr.len() - 1];

        let args = Self::parse_arguments(args_str, data);
        FORMATTERS.call(&ctx, &name, &args)
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

        if let Some(v) = Self::eval_math(arg, data) {
            return serde_json::json!(v);
        }

        if let Some(value) = resolve_value(data, &arg) {
            return serde_json::json!(value);
        }

        Value::Null
    }

    fn eval_math(expr: &str, data: &Value) -> Option<f64> {
        let mut ctx: HashMapContext<DefaultNumericTypes> = HashMapContext::new();

        // Chỉ hỗ trợ object ở root
        if let Value::Object(map) = data {
            for (key, value) in map {
                match value {
                    Value::Number(n) => {
                        if let Some(v) = n.as_f64() {
                            let _ = ctx.set_value(key.clone(), EvalValue::from_float(v));
                        }
                    }
                    Value::Bool(v) => {
                        let _ = ctx.set_value(key.clone(), EvalValue::Boolean(*v));
                    }
                    Value::String(v) => {
                        let _ = ctx.set_value(key.clone(), EvalValue::String(v.clone()));
                    }
                    _ => {}
                }
            }
        }
        eval_number_with_context(expr, &ctx).ok()
    }

    pub fn evaluate_visible_if(visible_if: Option<String>, data: &JsonValue) -> bool {
        let visible_if = match visible_if {
            Some(v) if !v.trim().is_empty() => v,
            _ => return true,
        };

        // Parse JSON
        let config: VisibleIfConfig = match serde_json::from_str(&visible_if) {
            Ok(v) => v,
            Err(_) => {
                eprintln!("visibleIf is not valid JSON: {}", visible_if);
                return true;
            }
        };

        let expression = match config.visible {
            Some(v) => v,
            None => return true,
        };

        // remove return ... ;
        let re_return = Regex::new(r"^return\s+").unwrap();

        let expression = re_return
            .replace(&expression, "")
            .replace(';', "")
            .trim()
            .to_string();

        // evalexpr dùng !
        let expression = expression.replace("not ", "!");

        let mut context = HashMapContext::<DefaultNumericTypes>::new();

        //-------------------------------------------------------
        // empty()
        //-------------------------------------------------------

        context
            .set_function(
                "empty".into(),
                Function::new(|argument| {
                    let value = argument.clone();

                    let result = match value {
                        EvalValue::Empty => true,

                        EvalValue::Int(v) => v == 0,

                        EvalValue::Float(v) => v == 0.0,

                        EvalValue::String(ref s) => s.trim().is_empty(),

                        EvalValue::Tuple(ref t) => t.is_empty(),

                        _ => false,
                    };

                    Ok(EvalValue::Boolean(result))
                }),
            )
            .unwrap();

        //-------------------------------------------------------
        // variables
        //-------------------------------------------------------

        for key in config.watch {
            let value = data.get(&key);

            let eval_value = match value {
                None | Some(JsonValue::Null) => EvalValue::Empty,

                Some(JsonValue::Bool(v)) => EvalValue::Boolean(*v),

                Some(JsonValue::Number(n)) => {
                    if let Some(i) = n.as_i64() {
                        EvalValue::Int(i)
                    } else {
                        EvalValue::Float(n.as_f64().unwrap_or(0.0))
                    }
                }

                Some(JsonValue::String(s)) => EvalValue::String(s.clone()),

                Some(JsonValue::Array(arr)) => EvalValue::Tuple(vec![EvalValue::Empty; arr.len()]),

                Some(JsonValue::Object(obj)) => {
                    if obj.is_empty() {
                        EvalValue::Empty
                    } else {
                        EvalValue::String(serde_json::to_string(obj).unwrap_or_default())
                    }
                }
            };

            context.set_value(key.into(), eval_value).unwrap();
        }

        match eval_boolean_with_context(&expression, &context) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("visibleIf evaluate error: {} {:?}", expression, e);
                true
            }
        }
    }
}
