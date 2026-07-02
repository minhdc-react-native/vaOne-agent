use crate::pdf::models::{RichStyle, TextRun};

pub struct RichTextParser;

impl RichStyle {
    pub fn from_flags(bold: bool, italic: bool, underline: bool) -> Self {
        match (bold, italic, underline) {
            (false, false, false) => RichStyle::Normal,
            (true, false, false) => RichStyle::B,
            (false, true, false) => RichStyle::I,
            (false, false, true) => RichStyle::U,
            (true, true, false) => RichStyle::BI,
            (true, false, true) => RichStyle::BU,
            (false, true, true) => RichStyle::IU,
            (true, true, true) => RichStyle::BIU,
        }
    }
}

impl RichTextParser {
    pub fn parse(input: &str) -> Vec<TextRun> {
        let mut runs = Vec::new();

        let mut bold = false;
        let mut italic = false;
        let mut underline = false;
        let mut color: Option<String> = None;

        let mut buffer = String::new();

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '<' {
                // Flush text trước khi đổi style
                if !buffer.is_empty() {
                    runs.push(TextRun {
                        text: std::mem::take(&mut buffer),
                        style: RichStyle::from_flags(bold, italic, underline),
                        color: color.clone(),
                        size: None,
                    });
                }

                let mut tag = String::new();

                i += 1;

                while i < chars.len() && chars[i] != '>' {
                    tag.push(chars[i]);
                    i += 1;
                }

                match tag.as_str() {
                    "b" => bold = true,
                    "/b" => bold = false,

                    "i" => italic = true,
                    "/i" => italic = false,

                    "u" => underline = true,
                    "/u" => underline = false,

                    "br" => buffer.push('\n'),

                    _ if tag.starts_with("color=") => {
                        color = Some(tag["color=".len()..].to_string());
                    }

                    "/color" => {
                        color = None;
                    }

                    _ => {}
                }
            } else {
                buffer.push(chars[i]);
            }

            i += 1;
        }

        if !buffer.is_empty() {
            runs.push(TextRun {
                text: buffer,
                style: RichStyle::from_flags(bold, italic, underline),
                color,
                size: None,
            });
        }

        runs
    }
}

impl RichStyle {
    pub fn bold(&self) -> bool {
        matches!(
            self,
            RichStyle::B | RichStyle::BI | RichStyle::BU | RichStyle::BIU
        )
    }

    pub fn italic(&self) -> bool {
        matches!(
            self,
            RichStyle::I | RichStyle::BI | RichStyle::IU | RichStyle::BIU
        )
    }

    pub fn underline(&self) -> bool {
        matches!(
            self,
            RichStyle::U | RichStyle::BU | RichStyle::IU | RichStyle::BIU
        )
    }
}
