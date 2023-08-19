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
    PropertyValue,
    PropertyComplete,
    OpenDoubleQuotes { raw: String, data: String },
    Whitespace(String),
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
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                        }
                    }
                } else {
                    self.tokens
                        .push_back(JsonToken::ObjectOpen(String::from(c)));
                    self.partial_tokens.push(JsonPartialToken::PropertyName);
                }
            }
            '}' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                        }
                    }
                } else {
                    todo!();
                }
            }
            '[' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                        }
                    }
                } else {
                    self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                    self.partial_tokens.push(JsonPartialToken::PropertyValue);
                }
            }
            ']' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                        }
                    }
                } else {
                    todo!();
                }
            }
            '"' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => {
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes {
                                    raw: String::from(c),
                                    data: String::new(),
                                });
                        }
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, data } => {
                            raw.push(c);
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::PropertyName => {
                                        self.tokens
                                            .push_back(JsonToken::PropertyName { raw, name: data });
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                    }
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens
                                            .push_back(JsonToken::StringValue { raw, value: data });
                                        self.partial_tokens
                                            .push(JsonPartialToken::PropertyComplete);
                                    }
                                    JsonPartialToken::PropertyComplete => todo!(),
                                    JsonPartialToken::OpenDoubleQuotes { raw: _, data: _ } => {
                                        todo!()
                                    }
                                    JsonPartialToken::Whitespace(_) => todo!(),
                                }
                            }
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes {
                                    raw: String::from(c),
                                    data: String::new(),
                                });
                        }
                    }
                } else {
                    todo!();
                }
            }
            ' ' | '\t' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => {
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                            self.partial_tokens
                                .push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::PropertyValue => {
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                            self.partial_tokens
                                .push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::PropertyComplete => {
                            self.partial_tokens.push(JsonPartialToken::PropertyComplete);
                            self.partial_tokens
                                .push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data })
                        }
                        JsonPartialToken::Whitespace(mut whitespace) => {
                            whitespace.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::Whitespace(whitespace));
                        }
                    }
                } else {
                    todo!();
                }
            }
            ':' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => {
                            self.partial_tokens.push(JsonPartialToken::PropertyValue)
                        }
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data })
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                            self.tokens
                                .push_back(JsonToken::KeyValueDelimiter(String::from(c)));
                        }
                    }
                } else {
                    todo!();
                }
            }
            '\n' => {
                self.tokens.push_back(JsonToken::NewLine(String::from(c)));
            }
            _ => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!("{}", c),
                        JsonPartialToken::PropertyValue => todo!("{}", c),
                        JsonPartialToken::PropertyComplete => todo!("{}", c),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens
                                .push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::Whitespace(_) => todo!("{}", c),
                    }
                } else {
                    todo!();
                }
            }
        }
    }
}
