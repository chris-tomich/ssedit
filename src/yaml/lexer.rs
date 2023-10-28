use std::collections::VecDeque;

use strum_macros::Display;

#[derive(Display, PartialEq)]
pub enum YamlToken {
    PropertyName { raw: String, name: String },
    BooleanValue { raw: String, value: bool },
    StringValue { raw: String, value: String },
    IntegerValue { raw: String, value: isize },
    FloatValue { raw: String, value: f64 },
    NullValue(String),
    ObjectOpen(String),
    ObjectClose(String),
    ArrayOpen(String),
    ArrayClose(String),
    Whitespace(String),
    NewLine(String),
    ArrayItemDelimiter(String),
    PropertyDelimiter(String),
    KeyValueDelimiter(String),
    Content(String),
    Paragraph(String),
    Line(String),
    ParagraphBreak(String),
    Alias(String),
    Dereference(String),
    Comment(String),
    YamlStart(String),
}

pub enum YamlPartialToken {
    Array,
    Object,
    PropertyName,
    PropertyValue,
    ArrayValue,
    BooleanValue { raw: String, value: bool },
    StringValue { raw: String, value: String },
    NullValue { raw: String },
    UndefinedValue { raw: String },
    Root,
    NumberValue(String),
    Whitespace(String),
    Content(String),
    Paragraph(String),
    Line(String),
    ParagraphBreak(String),
    Alias(String),
    Dereference(String),
    Comment(String),
    YamlStart(String),
}

pub enum YamlStreamStatus {
    None,
    Token(YamlToken),
}

pub struct YamlStreamLexer {
    tokens: VecDeque<YamlToken>,
    partial_tokens: Vec<YamlPartialToken>,
    is_error: bool,
}

impl YamlStreamLexer {
    pub fn new() -> YamlStreamLexer {
        let mut partial_tokens = Vec::new();
        partial_tokens.push(YamlPartialToken::Root);

        YamlStreamLexer {
            tokens: VecDeque::new(),
            partial_tokens,
            is_error: false,
        }
    }

    pub fn pop_token(&mut self) -> YamlStreamStatus {
        match self.tokens.pop_front() {
            Some(status) => YamlStreamStatus::Token(status),
            None => YamlStreamStatus::None,
        }
    }

    pub fn push_char(&mut self, c: char) -> Result<(), &str> {
        match c {
            '-' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        YamlPartialToken::Array => todo!(),
                        YamlPartialToken::Object => todo!(),
                        YamlPartialToken::PropertyName => todo!(),
                        YamlPartialToken::PropertyValue => todo!(),
                        YamlPartialToken::ArrayValue => todo!(),
                        YamlPartialToken::BooleanValue { raw, value } => todo!(),
                        YamlPartialToken::StringValue { raw, value } => todo!(),
                        YamlPartialToken::NullValue { raw } => todo!(),
                        YamlPartialToken::UndefinedValue { raw } => todo!(),
                        YamlPartialToken::Root => self.partial_tokens.push(YamlPartialToken::YamlStart(String::from(c))),
                        YamlPartialToken::NumberValue(_) => todo!(),
                        YamlPartialToken::Whitespace(_) => todo!(),
                        YamlPartialToken::Content(_) => todo!(),
                        YamlPartialToken::Paragraph(_) => todo!(),
                        YamlPartialToken::Line(_) => todo!(),
                        YamlPartialToken::ParagraphBreak(_) => todo!(),
                        YamlPartialToken::Alias(_) => todo!(),
                        YamlPartialToken::Dereference(_) => todo!(),
                        YamlPartialToken::Comment(_) => todo!(),
                        YamlPartialToken::YamlStart(mut raw) => {
                            raw.push(c);
                            self.partial_tokens.push(YamlPartialToken::YamlStart(raw));
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            '\n' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        YamlPartialToken::Array => todo!(),
                        YamlPartialToken::Object => todo!(),
                        YamlPartialToken::PropertyName => todo!(),
                        YamlPartialToken::PropertyValue => todo!(),
                        YamlPartialToken::ArrayValue => todo!(),
                        YamlPartialToken::BooleanValue { raw, value } => todo!(),
                        YamlPartialToken::StringValue { raw, value } => todo!(),
                        YamlPartialToken::NullValue { raw } => todo!(),
                        YamlPartialToken::UndefinedValue { raw } => todo!(),
                        YamlPartialToken::Root => {
                            self.tokens.push_back(YamlToken::NewLine(String::from(c)));
                            self.partial_tokens.push(YamlPartialToken::Root);
                        }
                        YamlPartialToken::NumberValue(_) => todo!(),
                        YamlPartialToken::Whitespace(_) => todo!(),
                        YamlPartialToken::Content(_) => todo!(),
                        YamlPartialToken::Paragraph(_) => todo!(),
                        YamlPartialToken::Line(_) => todo!(),
                        YamlPartialToken::ParagraphBreak(_) => todo!(),
                        YamlPartialToken::Alias(_) => todo!(),
                        YamlPartialToken::Dereference(_) => todo!(),
                        YamlPartialToken::Comment(_) => todo!(),
                        YamlPartialToken::YamlStart(raw) => {
                            self.tokens.push_back(YamlToken::YamlStart(raw));
                            self.partial_tokens.push(YamlPartialToken::Root);
                        }
                    }
                } else {
                    self.is_error = true;
                }

                self.tokens.push_back(YamlToken::NewLine(String::from(c)));
            }
            _ => {}
        }

        Ok(())
    }
}
