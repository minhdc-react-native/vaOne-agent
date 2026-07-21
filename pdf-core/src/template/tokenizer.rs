#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    OpenTag(String),
    CloseTag(String),
    Expression(String),
}

pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize(input: &str) -> Vec<Token> {
        let chars: Vec<char> = input.chars().collect();
        let mut tokens = Vec::new();

        let mut i = 0;
        let mut buffer = String::new();

        while i < chars.len() {
            match chars[i] {
                '<' => {
                    // flush text
                    if !buffer.is_empty() {
                        tokens.push(Token::Text(std::mem::take(&mut buffer)));
                    }

                    let mut tag = String::new();

                    i += 1;

                    while i < chars.len() && chars[i] != '>' {
                        tag.push(chars[i]);
                        i += 1;
                    }

                    if let Some(name) = tag.strip_prefix('/') {
                        tokens.push(Token::CloseTag(name.trim().to_string()));
                    } else {
                        tokens.push(Token::OpenTag(tag.trim().to_string()));
                    }
                }

                '{' => {
                    // flush text
                    if !buffer.is_empty() {
                        tokens.push(Token::Text(std::mem::take(&mut buffer)));
                    }

                    let mut expr = String::new();

                    i += 1;

                    while i < chars.len() && chars[i] != '}' {
                        expr.push(chars[i]);
                        i += 1;
                    }

                    tokens.push(Token::Expression(expr.trim().to_string()));
                }
                c => {
                    buffer.push(c);
                }
            }

            i += 1;
        }

        if !buffer.is_empty() {
            tokens.push(Token::Text(buffer));
        }

        tokens
    }
}
