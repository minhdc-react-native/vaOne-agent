use crate::utils::bind_content;
use crate::{layout::TextLayout, models::*};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicContent {
    pub watch: Vec<String>,
    pub fn_text: String,
}

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
                        // Mặc định
                        let mut content = text.content.clone();
                        let mut watch = Vec::new();

                        // Nếu content là DynamicContent
                        if let Ok(dynamic) = serde_json::from_str::<DynamicContent>(&text.content) {
                            content = dynamic.fn_text;
                            watch = dynamic.watch;
                        }

                        let context = TextLayout::build_context(data, field, "value", &watch);

                        text.content = bind_content(&content, &context);
                    }
                }
            }
            _ => {}
        }
    }
    report
}
