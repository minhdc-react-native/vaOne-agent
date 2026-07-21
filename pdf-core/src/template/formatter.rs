use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;

pub type FormatterFn = fn(&FormatterContext, &[Value]) -> Result<String>;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime};

use once_cell::sync::Lazy;

use crate::template::models::FormatterContext;

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

    pub fn call(&self, ctx: &FormatterContext, name: &str, args: &[Value]) -> Result<String> {
        match self.map.get(name) {
            Some(f) => f(ctx, args),
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

fn format_date(_ctx: &FormatterContext, args: &[Value]) -> Result<String> {
    let value = arg_string(args, 0);

    let format = if args.len() > 1 {
        arg_string(args, 1)
    } else {
        "dd/MM/yyyy".to_string()
    };

    let chrono_format = format
        .replace("yyyy", "%Y")
        .replace("MM", "%m")
        .replace("dd", "%d")
        .replace("HH", "%H")
        .replace("mm", "%M")
        .replace("ss", "%S");

    // ISO 8601: 2026-03-04T17:00:00Z
    if let Ok(dt) = DateTime::parse_from_rfc3339(&value) {
        return Ok(dt.format(&chrono_format).to_string());
    }

    // yyyy-MM-dd
    if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
        return Ok(date.format(&chrono_format).to_string());
    }

    Err(anyhow!("Invalid date: {}", value))
}

fn date_month_year(_ctx: &FormatterContext, args: &[Value]) -> Result<String> {
    let value = arg_string(args, 0);
    let format = args.get(1).and_then(|v| v.as_str()).unwrap_or("");

    fn format_date(d: NaiveDate, format: &str) -> String {
        if format.is_empty() {
            return format!(
                "Ngày {:02} tháng {:02} năm {}",
                d.day(),
                d.month(),
                d.year()
            );
        }

        // Chuyển format kiểu .NET/Java -> chrono
        let chrono_format = format
            .replace("dd", "%d")
            .replace("MM", "%m")
            .replace("yyyy", "%Y")
            .replace("yy", "%y");

        d.format(&chrono_format).to_string()
    }

    // 2026-03-04T17:00:00Z
    if let Ok(dt) = DateTime::parse_from_rfc3339(&value) {
        return Ok(format_date(dt.date_naive(), format));
    }

    // 2026-03-04T17:00:00
    if let Ok(dt) = NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S") {
        return Ok(format_date(dt.date(), format));
    }

    // 2026-03-04
    if let Ok(d) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
        return Ok(format_date(d, format));
    }

    Err(anyhow!("Invalid date: {}", value))
}

fn format_number(ctx: &FormatterContext, args: &[Value]) -> Result<String> {
    let value = arg_f64(args, 0);

    let format = args.get(1).and_then(|v| v.as_str()).unwrap_or("TIEN");

    let decimal_places = match format {
        "GIA" => ctx.decimal.local_unit_price_decimal_places,
        "GIA_NT" => ctx.decimal.foreign_unit_price_decimal_places,
        "SLG" => ctx.decimal.quantity_decimal_places,
        "TIEN" => ctx.decimal.local_currency_decimal_places,
        "TIEN_NT" => ctx.decimal.foreign_currency_decimal_places,
        "PT" => ctx.decimal.ratio_decimal_places,
        "EXCHANGE_RATE" => ctx.decimal.exchange_rate_decimal_places,
        _ => ctx.decimal.local_currency_decimal_places,
    };

    Ok(format_decimal(
        value,
        decimal_places,
        &ctx.decimal.thousand_separator,
        &ctx.decimal.decimal_separator,
    ))
}

fn format_decimal(
    value: f64,
    decimal_places: usize,
    thousand_separator: &str,
    decimal_separator: &str,
) -> String {
    let negative = value < 0.0;

    let value = value.abs();

    let text = format!("{:.*}", decimal_places, value);

    let mut parts = text.split('.');

    let int_part = parts.next().unwrap_or("0");
    let frac_part = parts.next();

    // format phần nguyên
    let mut int_result = String::new();

    for (i, ch) in int_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            int_result.insert_str(0, thousand_separator);
        }

        int_result.insert(0, ch);
    }

    let mut result = if decimal_places > 0 {
        format!(
            "{}{}{}",
            int_result,
            decimal_separator,
            frac_part.unwrap_or("")
        )
    } else {
        int_result
    };

    if negative {
        result.insert(0, '-');
    }

    result
}

fn money_to_words(ctx: &FormatterContext, args: &[Value]) -> Result<String> {
    let value = arg_f64(args, 0);

    match ctx.lang.as_str() {
        "vi" => Ok(number_to_vietnamese(ctx, value)),
        "en" => Ok(number_to_english(ctx, value)),
        _ => Ok(number_to_vietnamese(ctx, value)),
    }
}

fn number_to_vietnamese(ctx: &FormatterContext, value: f64) -> String {
    let negative = value < 0.0;
    let value = value.abs();

    let integer = value.trunc() as i64;

    let decimal = if ctx.currency.decimal_conversion_rate > 1 {
        ((value.fract() * ctx.currency.decimal_conversion_rate as f64).round()) as i64
    } else {
        0
    };

    let mut result = String::new();

    if negative {
        result.push_str("Âm ");
    }

    result.push_str(&read_number(integer));
    result.push(' ');
    result.push_str(&ctx.currency.currency_name_vn);

    if decimal > 0 {
        result.push(' ');
        result.push_str(&ctx.currency.separator_vn);
        result.push(' ');
        result.push_str(&read_number(decimal));
        result.push(' ');
        result.push_str(&ctx.currency.decimal_name_vn);
    }

    let mut chars = result.chars();

    if let Some(first) = chars.next() {
        first.to_uppercase().collect::<String>() + chars.as_str()
    } else {
        result
    }
}

fn read_number(mut num: i64) -> String {
    if num == 0 {
        return "không".to_string();
    }

    const DIGITS: [&str; 10] = [
        "không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín",
    ];

    const UNITS: [&str; 7] = ["", "nghìn", "triệu", "tỷ", "nghìn tỷ", "triệu tỷ", "tỷ tỷ"];

    fn read_block(n: i64, full: bool) -> String {
        const DIGITS: [&str; 10] = [
            "không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín",
        ];

        let hundred = n / 100;
        let ten = (n % 100) / 10;
        let one = n % 10;

        let mut s = String::new();

        if hundred > 0 || full {
            s.push_str(DIGITS[hundred as usize]);
            s.push_str(" trăm");
        }

        match ten {
            0 => {
                if one > 0 {
                    if hundred > 0 || full {
                        s.push_str(" lẻ");
                    }

                    s.push(' ');
                    s.push_str(DIGITS[one as usize]);
                }
            }

            1 => {
                s.push_str(" mười");

                match one {
                    0 => {}
                    5 => s.push_str(" lăm"),
                    _ => {
                        s.push(' ');
                        s.push_str(DIGITS[one as usize]);
                    }
                }
            }

            _ => {
                if ten > 1 {
                    s.push(' ');
                    s.push_str(DIGITS[ten as usize]);
                    s.push_str(" mươi");

                    match one {
                        0 => {}
                        1 => s.push_str(" mốt"),
                        4 => s.push_str(" tư"),
                        5 => s.push_str(" lăm"),
                        _ => {
                            s.push(' ');
                            s.push_str(DIGITS[one as usize]);
                        }
                    }
                }
            }
        }

        s.trim().to_string()
    }

    let mut blocks = Vec::new();

    while num > 0 {
        blocks.push(num % 1000);
        num /= 1000;
    }

    let mut parts = Vec::new();

    for i in (0..blocks.len()).rev() {
        let block = blocks[i];

        if block == 0 {
            continue;
        }

        let full = i != blocks.len() - 1 && block < 100;

        let mut part = read_block(block, full);

        if !UNITS[i].is_empty() {
            part.push(' ');
            part.push_str(UNITS[i]);
        }

        parts.push(part);
    }

    parts.join(" ")
}

fn number_to_english(ctx: &FormatterContext, value: f64) -> String {
    let negative = value < 0.0;
    let value = value.abs();

    let integer = value.trunc() as i64;

    let decimal = if ctx.currency.decimal_conversion_rate > 1 {
        ((value.fract() * ctx.currency.decimal_conversion_rate as f64).round()) as i64
    } else {
        0
    };

    let mut result = String::new();

    if negative {
        result.push_str("Minus ");
    }

    result.push_str(&read_number_en(integer));
    result.push(' ');
    result.push_str(&plural(&ctx.currency.currency_name_en, integer));

    if decimal > 0 {
        result.push(' ');
        result.push_str(&ctx.currency.separator_en);
        result.push(' ');
        result.push_str(&read_number_en(decimal));
        result.push(' ');
        result.push_str(&plural(&ctx.currency.decimal_name_en, decimal));
    }

    let mut chars = result.chars();

    if let Some(first) = chars.next() {
        first.to_uppercase().collect::<String>() + chars.as_str()
    } else {
        result
    }
}

fn read_number_en(mut num: i64) -> String {
    if num == 0 {
        return "zero".to_string();
    }

    const ONES: [&str; 20] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];

    const TENS: [&str; 10] = [
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];

    const UNITS: [&str; 7] = [
        "",
        "thousand",
        "million",
        "billion",
        "trillion",
        "quadrillion",
        "quintillion",
    ];

    fn read_block(n: i64) -> String {
        const ONES: [&str; 20] = [
            "zero",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "seventeen",
            "eighteen",
            "nineteen",
        ];

        const TENS: [&str; 10] = [
            "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];

        let hundred = n / 100;
        let remain = n % 100;

        let mut parts = Vec::new();

        if hundred > 0 {
            parts.push(format!("{} hundred", ONES[hundred as usize]));

            // British English
            if remain > 0 {
                parts.push("and".to_string());
            }
        }

        if remain > 0 {
            if remain < 20 {
                parts.push(ONES[remain as usize].to_string());
            } else {
                let ten = remain / 10;
                let one = remain % 10;

                if one == 0 {
                    parts.push(TENS[ten as usize].to_string());
                } else {
                    parts.push(format!("{}-{}", TENS[ten as usize], ONES[one as usize]));
                }
            }
        }

        parts.join(" ")
    }

    let mut blocks = Vec::new();

    while num > 0 {
        blocks.push(num % 1000);
        num /= 1000;
    }

    let mut parts = Vec::new();

    for i in (0..blocks.len()).rev() {
        let block = blocks[i];

        if block == 0 {
            continue;
        }

        let mut part = read_block(block);

        if !UNITS[i].is_empty() {
            part.push(' ');
            part.push_str(UNITS[i]);
        }

        parts.push(part);
    }

    parts.join(" ")
}

fn plural(word: &str, value: i64) -> String {
    if value == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

pub static FORMATTERS: Lazy<FormatterRegistry> = Lazy::new(FormatterRegistry::new);
