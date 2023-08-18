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
    PropertyName,
    OpenDoubleQuotes { raw: String, data: String },
}

pub enum JsonStreamStatus {
    None,
    Token(JsonToken),
    Finish,
}

pub struct JsonStreamLexer {
    tokens: VecDeque<JsonToken>,
    partial_tokens: Vec<JsonPartialToken>,
}

impl JsonStreamLexer {
    pub fn new() -> JsonStreamLexer {
        JsonStreamLexer {
            tokens: VecDeque::new(),
            partial_tokens: Vec::new(),
        }
    }

    pub fn pop_token(&mut self) -> JsonStreamStatus {
        match self.tokens.pop_front() {
            Some(status) => JsonStreamStatus::Token(status),
            None => JsonStreamStatus::None,
        }
    }

    pub fn push_char(&mut self, c: char) {
        match c {
            '{' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                    self.partial_tokens.push(JsonPartialToken::PropertyName);
                }
            }
            '}' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push_back(JsonToken::ObjectClose(String::from(c)));
                }
            }
            '[' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                }
            }
            ']' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                    }
                } else {
                    self.tokens.push_back(JsonToken::ArrayClose(String::from(c)));
                }
            }
            '"' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => {
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw: String::from(c), data: String::new() });
                        }
                        JsonPartialToken::OpenDoubleQuotes { mut raw, data } => {
                            raw.push(c);
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::PropertyName => self.tokens.push_back(JsonToken::PropertyName { raw, name: data }),
                                    JsonPartialToken::OpenDoubleQuotes { raw: _, data: _ } => todo!(),
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => self.partial_tokens.push(JsonPartialToken::PropertyName),
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
