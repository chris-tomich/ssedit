use std::collections::VecDeque;

use strum_macros::Display;

#[derive(Display, PartialEq)]
pub enum JsonToken {
    PropertyName { raw: String, name: String },
    StringValue { raw: String, value: String },
    NumberValue { raw: String, value: isize },
    ObjectOpen(String),
    ObjectClose(String),
    ArrayOpen(String),
    ArrayClose(String),
    Whitespace(String),
    NewLine(String),
    PropertyDelimiter(String),
    KeyValueDelimiter(String),
}

pub enum JsonPartialToken {
    PropertyName { raw: String, data: String },
    OpenDoubleQuotes { raw: String, data: String },
}

pub struct JsonStreamLexer {
    tokens: Vec<JsonToken>,
    partial_tokens: Vec<JsonPartialToken>,
}

impl JsonStreamLexer {
    pub fn new() -> JsonStreamLexer {
        JsonStreamLexer {
            tokens: Vec::new(),
            partial_tokens: Vec::new(),
        }
    }

    pub fn push_char(&mut self, c: char) {
        match c {
            '{' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                        }
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push(JsonToken::ObjectOpen(String::from(c)));
                }
            }
            '}' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName { raw, data } => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push(JsonToken::ObjectClose(String::from(c)));
                }
            }
            '[' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName { raw, data } => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push(JsonToken::ArrayOpen(String::from(c)));
                }
            }
            ']' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName { raw, data } => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push(JsonToken::ArrayClose(String::from(c)));
                }
            }
            '"' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName { raw, data } => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, data } => {
                            raw.push(c);
                            self.tokens.push(JsonToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push(JsonToken::ArrayClose(String::from(c)));
                }
            }
            _ => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName { raw, data } => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, data } => {
                            raw.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                }
            }
        }
    }
}
