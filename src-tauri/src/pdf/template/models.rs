use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatterContext {
    pub lang: String,
    pub decimal: DecimalConfig,
    pub currency: CurrencyConfig,
}

impl Default for FormatterContext {
    fn default() -> Self {
        Self {
            lang: "vi".to_string(),
            decimal: DecimalConfig::default(),
            currency: CurrencyConfig::default(),
        }
    }
}

impl Default for DecimalConfig {
    fn default() -> Self {
        Self {
            thousand_separator: ".".to_string(),
            decimal_separator: ",".to_string(),

            local_currency_decimal_places: 0,
            foreign_currency_decimal_places: 2,

            local_unit_price_decimal_places: 0,
            foreign_unit_price_decimal_places: 2,

            quantity_decimal_places: 2,
            exchange_rate_decimal_places: 2,
            ratio_decimal_places: 2,
        }
    }
}

impl Default for CurrencyConfig {
    fn default() -> Self {
        Self {
            code: "VND".to_string(),

            currency_name_en: "dong".to_string(),
            separator_en: "and".to_string(),
            decimal_name_en: "xu".to_string(),

            currency_name_vn: "đồng".to_string(),
            separator_vn: "và".to_string(),
            decimal_name_vn: "xu".to_string(),

            decimal_conversion_rate: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecimalConfig {
    pub thousand_separator: String,
    pub decimal_separator: String,
    pub local_currency_decimal_places: usize,
    pub foreign_currency_decimal_places: usize,
    pub local_unit_price_decimal_places: usize,
    pub foreign_unit_price_decimal_places: usize,
    pub quantity_decimal_places: usize,
    pub exchange_rate_decimal_places: usize,
    pub ratio_decimal_places: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyConfig {
    pub code: String,
    pub currency_name_en: String,
    pub separator_en: String,
    pub decimal_name_en: String,
    pub currency_name_vn: String,
    pub separator_vn: String,
    pub decimal_name_vn: String,
    pub decimal_conversion_rate: i64,
}

#[derive(Debug, Clone, Copy)]
enum NumberFormat {
    Price,
    ForeignPrice,
    Quantity,
    Money,
    ForeignMoney,
    Percent,
    ExchangeRate,
}

impl NumberFormat {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "GIA" => Some(Self::Price),
            "GIA_NT" => Some(Self::ForeignPrice),
            "SLG" => Some(Self::Quantity),
            "TIEN" => Some(Self::Money),
            "TIEN_NT" => Some(Self::ForeignMoney),
            "PT" => Some(Self::Percent),
            "EXCHANGE_RATE" => Some(Self::ExchangeRate),
            _ => None,
        }
    }
}
