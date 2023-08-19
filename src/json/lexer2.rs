use std::collections::VecDeque;

use strum_macros::Display;

#[derive(Display, PartialEq)]
pub enum JsonToken {
    PropertyName { raw: String, name: String },
    StringValue { raw: String, value: String },
    IntegerValue { raw: String, value: isize },
    FloatValue { raw: String, value: f64 },
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
    NumberValue(String),
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
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(_) => todo!(),
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
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
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(_) => todo!(),
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
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(_) => todo!(),
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
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(_) => todo!(),
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
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes {
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
                                        self.tokens.push_back(JsonToken::PropertyName { raw, name: data });
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                    }
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::StringValue { raw, value: data });
                                        self.partial_tokens.push(JsonPartialToken::PropertyComplete);
                                    }
                                    JsonPartialToken::PropertyComplete => todo!(),
                                    JsonPartialToken::OpenDoubleQuotes { raw: _, data: _ } => todo!(),
                                    JsonPartialToken::NumberValue(_) => todo!(),
                                    JsonPartialToken::Whitespace(_) => todo!(),
                                }
                            } else {
                                todo!();
                            }
                        }
                        JsonPartialToken::NumberValue(_) => todo!(),
                        JsonPartialToken::Whitespace(whitespace) => {
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::PropertyName => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.partial_tokens.push(JsonPartialToken::PropertyName);
                                        self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes {
                                            raw: String::from(c),
                                            data: String::new(),
                                        });
                                    }
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                        self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes {
                                            raw: String::from(c),
                                            data: String::new(),
                                        });
                                    }
                                    JsonPartialToken::PropertyComplete => todo!(),
                                    JsonPartialToken::OpenDoubleQuotes { raw: _, data: _ } => todo!(),
                                    JsonPartialToken::NumberValue(_) => todo!(),
                                    JsonPartialToken::Whitespace(_) => todo!(),
                                }
                            } else {
                                todo!();
                            }
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
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::PropertyValue => {
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::PropertyComplete => {
                            self.partial_tokens.push(JsonPartialToken::PropertyComplete);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(raw_number) => {
                            if raw_number.contains(".") {
                                if let Ok(number) = raw_number.as_str().parse::<f64>() {
                                    self.tokens.push_back(JsonToken::FloatValue { raw: raw_number, value: number });
                                    self.partial_tokens.push(JsonPartialToken::PropertyComplete);
                                }
                            } else {
                                if let Ok(number) = raw_number.as_str().parse::<isize>() {
                                    self.tokens.push_back(JsonToken::IntegerValue { raw: raw_number, value: number });
                                    self.partial_tokens.push(JsonPartialToken::PropertyComplete);
                                }
                            }
                        }
                        JsonPartialToken::Whitespace(mut whitespace) => {
                            whitespace.push(c);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(whitespace));
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
                            self.tokens.push_back(JsonToken::KeyValueDelimiter(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                        }
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(_) => todo!(),
                        JsonPartialToken::Whitespace(whitespace) => {
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::PropertyName => todo!(),
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.tokens.push_back(JsonToken::KeyValueDelimiter(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                    }
                                    JsonPartialToken::PropertyComplete => todo!(),
                                    JsonPartialToken::OpenDoubleQuotes { raw: _, data: _ } => todo!(),
                                    JsonPartialToken::NumberValue(_) => todo!(),
                                    JsonPartialToken::Whitespace(_) => todo!(),
                                }
                            } else {
                                todo!();
                            }
                        }
                    }
                } else {
                    todo!();
                }
            }
            '\n' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::PropertyComplete => self.partial_tokens.push(JsonPartialToken::PropertyComplete),
                        JsonPartialToken::OpenDoubleQuotes { raw: _, data: _ } => todo!(),
                        JsonPartialToken::NumberValue(raw_number) => {
                            if raw_number.contains(".") {
                                if let Ok(number) = raw_number.as_str().parse::<f64>() {
                                    self.tokens.push_back(JsonToken::FloatValue { raw: raw_number, value: number });
                                    self.partial_tokens.push(JsonPartialToken::PropertyComplete);
                                }
                            } else {
                                if let Ok(number) = raw_number.as_str().parse::<isize>() {
                                    self.tokens.push_back(JsonToken::IntegerValue { raw: raw_number, value: number });
                                    self.partial_tokens.push(JsonPartialToken::PropertyComplete);
                                }
                            }
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                        }
                    }
                } else {
                    todo!();
                }

                self.tokens.push_back(JsonToken::NewLine(String::from(c)));
            }
            '.' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(mut number) => {
                            if number.contains(".") {
                                panic!("malformed number");
                            } else {
                                number.push(c);
                                self.partial_tokens.push(JsonPartialToken::NumberValue(number));
                            }
                        }
                        JsonPartialToken::Whitespace(_) => todo!(),
                    }
                } else {
                    todo!();
                }
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c))),
                        JsonPartialToken::PropertyComplete => todo!(),
                        JsonPartialToken::OpenDoubleQuotes { mut raw, mut data } => {
                            raw.push(c);
                            data.push(c);
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(mut number) => {
                            number.push(c);
                            self.partial_tokens.push(JsonPartialToken::NumberValue(number));
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::PropertyName => todo!(),
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c)));
                                    }
                                    JsonPartialToken::PropertyComplete => todo!(),
                                    JsonPartialToken::OpenDoubleQuotes { raw: _, data: _ } => todo!(),
                                    JsonPartialToken::NumberValue(_) => todo!(),
                                    JsonPartialToken::Whitespace(_) => todo!(),
                                }
                            } else {
                                todo!();
                            }
                        }
                    }
                } else {
                    todo!();
                }
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
                            self.partial_tokens.push(JsonPartialToken::OpenDoubleQuotes { raw, data });
                        }
                        JsonPartialToken::NumberValue(_) => todo!(),
                        JsonPartialToken::Whitespace(_) => todo!("{}", c),
                    }
                } else {
                    todo!();
                }
            }
        }

        if self.partial_tokens.len() == 0 {
            println!("empty '{}'", c);
        }
    }
}
