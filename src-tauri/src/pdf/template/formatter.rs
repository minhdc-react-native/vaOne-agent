use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

pub type FormatterFn = fn(&[Value]) -> Result<String>;
use num_format::{Locale, ToFormattedString};
use once_cell::sync::Lazy;

pub struct FormatterRegistry {
    map: HashMap<String, FormatterFn>,
}

impl FormatterRegistry {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        map.insert("moneyToWords".to_string(), money_to_words as FormatterFn);
        map.insert("formatDate".to_string(), format_date as FormatterFn);
        map.insert("dateMonthYear".to_string(), date_month_year as FormatterFn);
        map.insert("formatNumber".to_string(), format_number as FormatterFn);

        Self { map }
    }

    pub fn call(&self, name: &str, args: &[Value]) -> Result<String> {
        match self.map.get(name) {
            Some(f) => f(args),
            None => Err(anyhow!("Formatter '{}' not found", name)),
        }
    }
}

fn arg_string(args: &[Value], index: usize) -> String {
    args.get(index)
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else {
                Some(v.to_string())
            }
        })
        .unwrap_or_default()
}

fn arg_f64(args: &[Value], index: usize) -> f64 {
    args.get(index).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

fn money_to_words(args: &[Value]) -> Result<String> {
    let value = arg_f64(args, 0);

    Ok(number_to_vietnamese(value as i64))
}

fn format_date(args: &[Value]) -> Result<String> {
    let value = arg_string(args, 0);

    let format = if args.len() > 1 {
        arg_string(args, 1)
    } else {
        "dd/MM/yyyy".to_string()
    };

    // TODO:
    // sử dụng chrono để format giống React
    Ok(format!("{} ({})", value, format))
}

fn date_month_year(args: &[Value]) -> Result<String> {
    let value = arg_string(args, 0);

    let format = if args.len() > 1 {
        arg_string(args, 1)
    } else {
        "Ngày dd tháng MM năm yyyy".to_string()
    };

    Ok(format!("{} ({})", value, format))
}

fn format_number(args: &[Value]) -> Result<String> {
    let value = arg_f64(args, 0);

    let format = if args.len() > 1 {
        arg_string(args, 1)
    } else {
        "TIEN".to_string()
    };

    match format.as_str() {
        "TIEN" => {
            let n = value.round() as i64;
            Ok(n.to_formatted_string(&Locale::en))
        }
        _ => Ok(value.to_string()),
    }
}

fn number_to_vietnamese(mut num: i64) -> String {
    if num == 0 {
        return "Không đồng".to_string();
    }

    let dv = ["", "nghìn", "triệu", "tỷ", "nghìn tỷ", "triệu tỷ"];

    let cs = [
        "không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín",
    ];

    fn read3(n: i64, cs: &[&str]) -> String {
        let tr = n / 100;
        let ch = (n % 100) / 10;
        let dv = n % 10;

        let mut s = String::new();

        if tr > 0 {
            s.push_str(cs[tr as usize]);
            s.push_str(" trăm");

            if ch == 0 && dv > 0 {
                s.push_str(" lẻ");
            }
        }

        if ch > 1 {
            s.push(' ');
            s.push_str(cs[ch as usize]);
            s.push_str(" mươi");

            match dv {
                1 => s.push_str(" mốt"),
                5 => s.push_str(" lăm"),
                d if d > 0 => {
                    s.push(' ');
                    s.push_str(cs[d as usize]);
                }
                _ => {}
            }
        } else if ch == 1 {
            s.push_str(" mười");

            match dv {
                1 => s.push_str(" một"),
                5 => s.push_str(" lăm"),
                d if d > 0 => {
                    s.push(' ');
                    s.push_str(cs[d as usize]);
                }
                _ => {}
            }
        } else if dv > 0 {
            if tr > 0 {
                s.push(' ');
            }

            s.push_str(cs[dv as usize]);
        }

        s
    }

    let mut parts = Vec::new();
    let mut i = 0;

    while num > 0 {
        let block = num % 1000;

        if block > 0 {
            parts.push(format!("{} {}", read3(block, &cs), dv[i]));
        }

        num /= 1000;
        i += 1;
    }

    let mut result = parts.into_iter().rev().collect::<Vec<_>>().join(" ");

    result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    let mut chars = result.chars();

    if let Some(first) = chars.next() {
        result = first.to_uppercase().collect::<String>() + chars.as_str();
    }

    result + " đồng"
}

pub static FORMATTERS: Lazy<FormatterRegistry> = Lazy::new(FormatterRegistry::new);
