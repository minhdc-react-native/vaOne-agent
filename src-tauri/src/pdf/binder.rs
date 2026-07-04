use crate::pdf::models::*;
use crate::pdf::utils::{bind_content, resolve_array};
use serde_json::{json, Value};

pub fn resolve_value(data: &Value, path: &str) -> Option<String> {
    let mut current = data;

    for key in path.split('.') {
        current = current.get(key)?;
    }

    match current {
        Value::Null => None,
        Value::String(v) => Some(v.clone()),
        Value::Number(v) => Some(v.to_string()),
        Value::Bool(v) => Some(v.to_string()),
        _ => Some(current.to_string()),
    }
}

pub fn bind_template(mut report: PdfTemplate, data: &serde_json::Value) -> PdfTemplate {
    for element in report.elements.iter_mut() {
        match element {
            Element::Text(text) => {
                if let Some(field) = &text.field_name {
                    if let Some(value) = resolve_value(data, field) {
                        let data_tmp: Value = json!({
                            "value": value
                        });
                        text.content = bind_content(&text.content, &data_tmp);
                    }
                }
            }
            Element::Table(table) => {
                if let Some(field) = &table.field_name {
                    if let Some(rows) = resolve_array(data, field) {}
                }
            }
        }
    }
    report
}
