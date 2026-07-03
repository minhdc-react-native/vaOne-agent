use serde_json::Value;

use crate::pdf::models::{TextRun, TextStyle};
use crate::pdf::template::evaluator::Evaluator;
use crate::pdf::template::tokenizer::Token;

pub struct Parser;

impl Parser {
    pub fn parse(tokens: &[Token], data: &Value) -> Vec<TextRun> {
        let mut runs = Vec::<TextRun>::new();

        let mut bold = false;
        let mut italic = false;
        let mut underline = false;

        for token in tokens {
            match token {
                Token::OpenTag(tag) => match tag.as_str() {
                    "b" => bold = true,
                    "i" => italic = true,
                    "u" => underline = true,

                    "bi" => {
                        bold = true;
                        italic = true;
                    }

                    "bu" => {
                        bold = true;
                        underline = true;
                    }

                    "iu" => {
                        italic = true;
                        underline = true;
                    }

                    "biu" => {
                        bold = true;
                        italic = true;
                        underline = true;
                    }

                    _ => {}
                },

                Token::CloseTag(tag) => match tag.as_str() {
                    "b" => bold = false,
                    "i" => italic = false,
                    "u" => underline = false,

                    "bi" => {
                        bold = false;
                        italic = false;
                    }

                    "bu" => {
                        bold = false;
                        underline = false;
                    }

                    "iu" => {
                        italic = false;
                        underline = false;
                    }

                    "biu" => {
                        bold = false;
                        italic = false;
                        underline = false;
                    }

                    _ => {}
                },

                Token::Text(text) => {
                    if !text.is_empty() {
                        runs.push(TextRun {
                            text: text.clone(),
                            style: TextStyle {
                                bold,
                                italic,
                                underline,
                                color: None,
                                font_size: None,
                            },
                            color: None,
                            size: None,
                        });
                    }
                }

                Token::Expression(expr) => {
                    let value = Evaluator::evaluate(expr, data).unwrap_or_default();

                    runs.push(TextRun {
                        text: value,
                        style: TextStyle {
                            bold,
                            italic,
                            underline,
                            color: None,
                            font_size: None,
                        },
                        color: None,
                        size: None,
                    });
                }
            }
        }

        runs
    }
}
