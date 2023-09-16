use std::collections::VecDeque;

use strum_macros::Display;

#[derive(Display, PartialEq)]
pub enum JsonToken {
    PropertyName { raw: String, name: String },
    BooleanValue { raw: String, value: bool },
    StringValue { raw: String, value: String },
    IntegerValue { raw: String, value: isize },
    FloatValue { raw: String, value: f64 },
    ObjectOpen(String),
    ObjectClose(String),
    ArrayOpen(String),
    ArrayClose(String),
    Whitespace(String),
    NewLine(String),
    ArrayItemDelimiter(String),
    PropertyDelimiter(String),
    KeyValueDelimiter(String),
}

pub enum JsonPartialToken {
    Array,
    Object,
    PropertyName,
    PropertyValue,
    ArrayValue,
    BooleanValue { raw: String, value: bool },
    StringValue { raw: String, value: String },
    Root,
    NumberValue(String),
    Whitespace(String),
}

pub enum JsonStreamStatus {
    None,
    Token(JsonToken),
}

pub struct JsonStreamLexer {
    tokens: VecDeque<JsonToken>,
    partial_tokens: Vec<JsonPartialToken>,
    current_line: String,
    is_error: bool,
}

impl JsonStreamLexer {
    pub fn new() -> JsonStreamLexer {
        let mut partial_tokens = Vec::new();
        partial_tokens.push(JsonPartialToken::Root);

        JsonStreamLexer {
            tokens: VecDeque::new(),
            partial_tokens,
            current_line: String::new(),
            is_error: false,
        }
    }

    pub fn close(&mut self) {
        while let Some(partial_token) = self.partial_tokens.pop() {
            match partial_token {
                JsonPartialToken::Array => {}
                JsonPartialToken::Object => {}
                JsonPartialToken::PropertyName => {}
                JsonPartialToken::PropertyValue => {}
                JsonPartialToken::ArrayValue => {}
                JsonPartialToken::BooleanValue { raw, value } => self.tokens.push_back(JsonToken::BooleanValue { raw, value }),
                JsonPartialToken::StringValue { raw, value } => self.tokens.push_back(JsonToken::StringValue { raw, value }),
                JsonPartialToken::Root => {}
                JsonPartialToken::NumberValue(raw_number) => {
                    if raw_number.contains(".") {
                        if let Ok(number) = raw_number.as_str().parse::<f64>() {
                            self.tokens.push_back(JsonToken::FloatValue { raw: raw_number, value: number });
                        }
                    } else {
                        if let Ok(number) = raw_number.as_str().parse::<isize>() {
                            self.tokens.push_back(JsonToken::IntegerValue { raw: raw_number, value: number });
                        }
                    }
                }
                JsonPartialToken::Whitespace(whitespace) => self.tokens.push_back(JsonToken::Whitespace(whitespace)),
            }
        }
    }

    pub fn pop_token(&mut self) -> JsonStreamStatus {
        match self.tokens.pop_front() {
            Some(status) => JsonStreamStatus::Token(status),
            None => JsonStreamStatus::None,
        }
    }

    pub fn push_char(&mut self, c: char) -> Result<(), &str> {
        match c {
            '\n' => {
                if !self.is_error {
                    self.current_line = String::new();
                } else {
                    return Err(&self.current_line);
                }
            }
            _ => self.current_line.push(c),
        }

        match c {
            '{' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => {
                            self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Array);
                            self.partial_tokens.push(JsonPartialToken::Object);
                        }
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => {
                            self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                            self.partial_tokens.push(JsonPartialToken::Object);
                        }
                        JsonPartialToken::ArrayValue => {
                            self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::ArrayValue);
                            self.partial_tokens.push(JsonPartialToken::Object);
                        }
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => {
                            self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Root);
                            self.partial_tokens.push(JsonPartialToken::Object);
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                        }
                        JsonPartialToken::NumberValue(_) => self.is_error = true,
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                            self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Object);
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            '}' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.is_error = true,
                        JsonPartialToken::Object => self.tokens.push_back(JsonToken::ObjectClose(String::from(c))),
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => self.tokens.push_back(JsonToken::ObjectClose(String::from(c))),
                        JsonPartialToken::ArrayValue => self.is_error = true,
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(raw_number) => {
                            if raw_number.contains(".") {
                                if let Ok(number) = raw_number.as_str().parse::<f64>() {
                                    self.tokens.push_back(JsonToken::FloatValue { raw: raw_number, value: number });
                                }
                            } else {
                                if let Ok(number) = raw_number.as_str().parse::<isize>() {
                                    self.tokens.push_back(JsonToken::IntegerValue { raw: raw_number, value: number });
                                }
                            }

                            self.tokens.push_back(JsonToken::ArrayClose(String::from(c)));

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Object => {}
                                    _ => self.is_error = true,
                                }
                            }
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                            self.tokens.push_back(JsonToken::ObjectClose(String::from(c)));

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Object => {}
                                    JsonPartialToken::PropertyValue => {
                                        if let Some(partial_token) = self.partial_tokens.pop() {
                                            match partial_token {
                                                JsonPartialToken::Object => {}
                                                _ => self.is_error = true,
                                            }
                                        }
                                    }
                                    _ => self.is_error = true,
                                }
                            }
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            '[' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => {
                            self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Array);
                            self.partial_tokens.push(JsonPartialToken::Array);
                        }
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => {
                            self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                            self.partial_tokens.push(JsonPartialToken::Array);
                        }
                        JsonPartialToken::ArrayValue => {
                            self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::ArrayValue);
                            self.partial_tokens.push(JsonPartialToken::Array);
                        }
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => {
                            self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Root);
                            self.partial_tokens.push(JsonPartialToken::Array);
                            self.partial_tokens.push(JsonPartialToken::ArrayValue);
                        }
                        JsonPartialToken::NumberValue(_) => self.is_error = true,
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {
                                        self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
                                    JsonPartialToken::Object => self.is_error = true,
                                    JsonPartialToken::PropertyName => self.is_error = true,
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
                                    JsonPartialToken::ArrayValue => {
                                        self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::ArrayValue);
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
                                    JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::StringValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::Root => {
                                        self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::Root);
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
                                    JsonPartialToken::NumberValue(_) => self.is_error = true,
                                    JsonPartialToken::Whitespace(_) => self.is_error = true,
                                }
                            } else {
                                self.is_error = true;
                            }
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            ']' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.tokens.push_back(JsonToken::ArrayClose(String::from(c))),
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => self.is_error = true,
                        JsonPartialToken::ArrayValue => self.tokens.push_back(JsonToken::ArrayClose(String::from(c))),
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(raw_number) => {
                            if raw_number.contains(".") {
                                if let Ok(number) = raw_number.as_str().parse::<f64>() {
                                    self.tokens.push_back(JsonToken::FloatValue { raw: raw_number, value: number });
                                }
                            } else {
                                if let Ok(number) = raw_number.as_str().parse::<isize>() {
                                    self.tokens.push_back(JsonToken::IntegerValue { raw: raw_number, value: number });
                                }
                            }

                            self.tokens.push_back(JsonToken::ArrayClose(String::from(c)));

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {}
                                    _ => self.is_error = true,
                                }
                            }
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                            self.tokens.push_back(JsonToken::ArrayClose(String::from(c)));

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {}
                                    _ => self.is_error = true,
                                }
                            }
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            '"' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => {
                            self.partial_tokens.push(JsonPartialToken::Array);
                            self.partial_tokens.push(JsonPartialToken::StringValue {
                                raw: String::from(c),
                                value: String::new(),
                            });
                        }
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => {
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                            self.partial_tokens.push(JsonPartialToken::StringValue {
                                raw: String::from(c),
                                value: String::new(),
                            });
                        }
                        JsonPartialToken::PropertyValue => {
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                            self.partial_tokens.push(JsonPartialToken::StringValue {
                                raw: String::from(c),
                                value: String::new(),
                            });
                        }
                        JsonPartialToken::ArrayValue => {
                            self.partial_tokens.push(JsonPartialToken::ArrayValue);
                            self.partial_tokens.push(JsonPartialToken::StringValue {
                                raw: String::from(c),
                                value: String::new(),
                            });
                        }
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, value } => {
                            raw.push(c);
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {
                                        self.tokens.push_back(JsonToken::StringValue { raw, value });
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
                                    JsonPartialToken::Object => {
                                        self.tokens.push_back(JsonToken::PropertyName { raw, name: value });
                                        self.partial_tokens.push(JsonPartialToken::Object);
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                    }
                                    JsonPartialToken::PropertyName => {
                                        self.tokens.push_back(JsonToken::PropertyName { raw, name: value });
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                    }
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::StringValue { raw, value });
                                    }
                                    JsonPartialToken::ArrayValue => {
                                        self.tokens.push_back(JsonToken::StringValue { raw, value });
                                    }
                                    JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::StringValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::Root => self.is_error = true,
                                    JsonPartialToken::NumberValue(_) => self.is_error = true,
                                    JsonPartialToken::Whitespace(_) => self.is_error = true,
                                }
                            } else {
                                self.is_error = true;
                            }
                        }
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(_) => self.is_error = true,
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                        self.partial_tokens.push(JsonPartialToken::StringValue {
                                            raw: String::from(c),
                                            value: String::new(),
                                        });
                                    }
                                    JsonPartialToken::Object => {
                                        self.partial_tokens.push(JsonPartialToken::Object);
                                        self.partial_tokens.push(JsonPartialToken::StringValue {
                                            raw: String::from(c),
                                            value: String::new(),
                                        });
                                    }
                                    JsonPartialToken::PropertyName => {
                                        self.partial_tokens.push(JsonPartialToken::PropertyName);
                                        self.partial_tokens.push(JsonPartialToken::StringValue {
                                            raw: String::from(c),
                                            value: String::new(),
                                        });
                                    }
                                    JsonPartialToken::PropertyValue => {
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                        self.partial_tokens.push(JsonPartialToken::StringValue {
                                            raw: String::from(c),
                                            value: String::new(),
                                        });
                                    }
                                    JsonPartialToken::ArrayValue => {
                                        self.partial_tokens.push(JsonPartialToken::ArrayValue);
                                        self.partial_tokens.push(JsonPartialToken::StringValue {
                                            raw: String::from(c),
                                            value: String::new(),
                                        });
                                    }
                                    JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::StringValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::Root => self.is_error = true,
                                    JsonPartialToken::NumberValue(_) => self.is_error = true,
                                    JsonPartialToken::Whitespace(_) => self.is_error = true,
                                }
                            } else {
                                self.is_error = true;
                            }
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            ' ' | '\t' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => {
                            self.partial_tokens.push(JsonPartialToken::Array);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::Object => {
                            self.partial_tokens.push(JsonPartialToken::Object);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::PropertyName => {
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::PropertyValue => {
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::ArrayValue => {
                            self.partial_tokens.push(JsonPartialToken::ArrayValue);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => {
                            self.partial_tokens.push(JsonPartialToken::Root);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                        }
                        JsonPartialToken::NumberValue(raw_number) => {
                            if raw_number.contains(".") {
                                if let Ok(number) = raw_number.as_str().parse::<f64>() {
                                    self.tokens.push_back(JsonToken::FloatValue { raw: raw_number, value: number });
                                }
                            } else {
                                if let Ok(number) = raw_number.as_str().parse::<isize>() {
                                    self.tokens.push_back(JsonToken::IntegerValue { raw: raw_number, value: number });
                                }
                            }

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                        self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                                    }
                                    JsonPartialToken::Object => {
                                        self.partial_tokens.push(JsonPartialToken::Object);
                                        self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                                    }
                                    JsonPartialToken::PropertyName => self.is_error = true,
                                    JsonPartialToken::PropertyValue => {
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                        self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                                    }
                                    JsonPartialToken::ArrayValue => {
                                        self.partial_tokens.push(JsonPartialToken::ArrayValue);
                                        self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                                    }
                                    JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::StringValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::Root => self.is_error = true,
                                    JsonPartialToken::NumberValue(_) => self.is_error = true,
                                    JsonPartialToken::Whitespace(_) => self.is_error = true,
                                }
                            } else {
                                self.is_error = true;
                            }
                        }
                        JsonPartialToken::Whitespace(mut whitespace) => {
                            whitespace.push(c);
                            self.partial_tokens.push(JsonPartialToken::Whitespace(whitespace));
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            ':' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.is_error = true,
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => {
                            self.tokens.push_back(JsonToken::KeyValueDelimiter(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                        }
                        JsonPartialToken::ArrayValue => self.is_error = true,
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(_) => self.is_error = true,
                        JsonPartialToken::Whitespace(whitespace) => {
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => self.is_error = true,
                                    JsonPartialToken::Object => self.is_error = true,
                                    JsonPartialToken::PropertyName => self.is_error = true,
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.tokens.push_back(JsonToken::KeyValueDelimiter(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                    }
                                    JsonPartialToken::ArrayValue => self.is_error = true,
                                    JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::StringValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::Root => self.is_error = true,
                                    JsonPartialToken::NumberValue(_) => self.is_error = true,
                                    JsonPartialToken::Whitespace(_) => self.is_error = true,
                                }
                            } else {
                                self.is_error = true;
                            }
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            ',' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => {
                            self.tokens.push_back(JsonToken::ArrayItemDelimiter(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Array);
                        }
                        JsonPartialToken::Object => {
                            self.tokens.push_back(JsonToken::PropertyDelimiter(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Object);
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                        }
                        JsonPartialToken::PropertyName => panic!("malformed property"),
                        JsonPartialToken::PropertyValue => self.tokens.push_back(JsonToken::PropertyDelimiter(String::from(c))),
                        JsonPartialToken::ArrayValue => self.tokens.push_back(JsonToken::ArrayItemDelimiter(String::from(c))),
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { raw: _, value: _ } => panic!("malformed string"),
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(raw_number) => {
                            if raw_number.contains(".") {
                                if let Ok(number) = raw_number.as_str().parse::<f64>() {
                                    self.tokens.push_back(JsonToken::FloatValue { raw: raw_number, value: number });
                                } else {
                                    panic!("malformed number");
                                }
                            } else {
                                if let Ok(number) = raw_number.as_str().parse::<isize>() {
                                    self.tokens.push_back(JsonToken::IntegerValue { raw: raw_number, value: number });
                                } else {
                                    panic!("malformed number");
                                }
                            }

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {
                                        self.tokens.push_back(JsonToken::ArrayItemDelimiter(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
                                    JsonPartialToken::Object => {
                                        self.tokens.push_back(JsonToken::PropertyDelimiter(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::Object);
                                    }
                                    JsonPartialToken::PropertyName => {
                                        self.tokens.push_back(JsonToken::PropertyDelimiter(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::PropertyName);
                                    }
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::PropertyDelimiter(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                    }
                                    JsonPartialToken::ArrayValue => {
                                        self.tokens.push_back(JsonToken::ArrayItemDelimiter(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::ArrayValue);
                                    }
                                    JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::StringValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::Root => self.is_error = true,
                                    JsonPartialToken::NumberValue(_) => self.is_error = true,
                                    JsonPartialToken::Whitespace(_) => self.is_error = true,
                                }
                            } else {
                                self.is_error = true;
                            }
                        }
                        JsonPartialToken::Whitespace(_) => self.is_error = true,
                    }
                } else {
                    self.is_error = true;
                }
            }
            '\n' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.partial_tokens.push(JsonPartialToken::Array),
                        JsonPartialToken::Object => self.partial_tokens.push(JsonPartialToken::Object),
                        JsonPartialToken::PropertyName => self.partial_tokens.push(JsonPartialToken::PropertyName),
                        JsonPartialToken::PropertyValue => self.partial_tokens.push(JsonPartialToken::PropertyValue),
                        JsonPartialToken::ArrayValue => self.partial_tokens.push(JsonPartialToken::ArrayValue),
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::Root => self.partial_tokens.push(JsonPartialToken::Root),
                        JsonPartialToken::NumberValue(raw_number) => {
                            if raw_number.contains(".") {
                                if let Ok(number) = raw_number.as_str().parse::<f64>() {
                                    self.tokens.push_back(JsonToken::FloatValue { raw: raw_number, value: number });
                                } else {
                                    panic!("malformed number");
                                }
                            } else {
                                if let Ok(number) = raw_number.as_str().parse::<isize>() {
                                    self.tokens.push_back(JsonToken::IntegerValue { raw: raw_number, value: number });
                                } else {
                                    panic!("malformed number");
                                }
                            }
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                        }
                    }
                } else {
                    self.is_error = true;
                }

                self.tokens.push_back(JsonToken::NewLine(String::from(c)));
            }
            '.' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.is_error = true,
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => self.is_error = true,
                        JsonPartialToken::ArrayValue => self.is_error = true,
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(mut number) => {
                            if number.contains(".") {
                                panic!("malformed number");
                            } else {
                                number.push(c);
                                self.partial_tokens.push(JsonPartialToken::NumberValue(number));
                            }
                        }
                        JsonPartialToken::Whitespace(_) => self.is_error = true,
                    }
                } else {
                    self.is_error = true;
                }
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => {
                            self.partial_tokens.push(JsonPartialToken::Array);
                            self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c)));
                        }
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c))),
                        JsonPartialToken::ArrayValue => self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c))),
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(mut number) => {
                            number.push(c);
                            self.partial_tokens.push(JsonPartialToken::NumberValue(number));
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                        self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c)));
                                    }
                                    JsonPartialToken::Object => self.is_error = true,
                                    JsonPartialToken::PropertyName => self.is_error = true,
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c)));
                                    }
                                    JsonPartialToken::ArrayValue => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c)));
                                    }
                                    JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::StringValue { raw: _, value: _ } => self.is_error = true,
                                    JsonPartialToken::Root => self.is_error = true,
                                    JsonPartialToken::NumberValue(_) => self.is_error = true,
                                    JsonPartialToken::Whitespace(_) => self.is_error = true,
                                }
                            } else {
                                self.is_error = true;
                            }
                        }
                    }
                } else {
                    self.is_error = true;
                }
            }
            't' | 'r' | 'u' | 'e' | 'f' | 'a' | 'l' | 's' | 'T' | 'R' | 'U' | 'E' | 'F' | 'A' | 'L' | 'S' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.is_error = true,
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => match c {
                            't' | 'T' => self.partial_tokens.push(JsonPartialToken::BooleanValue { raw: String::from(c), value: true }),
                            'f' | 'F' => self.partial_tokens.push(JsonPartialToken::BooleanValue { raw: String::from(c), value: false }),
                            _ => self.is_error = true,
                        },
                        JsonPartialToken::ArrayValue => match c {
                            't' | 'T' => self.partial_tokens.push(JsonPartialToken::BooleanValue { raw: String::from(c), value: true }),
                            'f' | 'F' => self.partial_tokens.push(JsonPartialToken::BooleanValue { raw: String::from(c), value: false }),
                            _ => self.is_error = true,
                        },
                        JsonPartialToken::BooleanValue { mut raw, value } => {
                            raw.push(c);
                            if c == 'e' || c == 'E' {
                                self.tokens.push_back(JsonToken::BooleanValue { raw, value });
                            } else {
                                self.partial_tokens.push(JsonPartialToken::BooleanValue { raw, value });
                            }
                        }
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(_) => self.is_error = true,
                        JsonPartialToken::Whitespace(_) => match c {
                            't' | 'T' => self.partial_tokens.push(JsonPartialToken::BooleanValue { raw: String::from(c), value: true }),
                            'f' | 'F' => self.partial_tokens.push(JsonPartialToken::BooleanValue { raw: String::from(c), value: false }),
                            _ => self.is_error = true,
                        },
                    }
                } else {
                    self.is_error = true;
                }
            }
            _ => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.is_error = true,
                        JsonPartialToken::Object => self.is_error = true,
                        JsonPartialToken::PropertyName => self.is_error = true,
                        JsonPartialToken::PropertyValue => self.is_error = true,
                        JsonPartialToken::ArrayValue => self.is_error = true,
                        JsonPartialToken::BooleanValue { raw: _, value: _ } => self.is_error = true,
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => self.is_error = true,
                        JsonPartialToken::NumberValue(_) => self.is_error = true,
                        JsonPartialToken::Whitespace(_) => self.is_error = true,
                    }
                } else {
                    self.is_error = true;
                }
            }
        }

        if self.partial_tokens.len() == 0 {
            println!("empty '{}'", c);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate lazy_static;

    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref TABBED_JSON_SAMPLE: &'static str = r#"{
	"id": "0001",
	"type": "donut",
	"name": "Cake",
	"ppu": 0.55,
	"style": [ "hole", "filled" ],
	"batters":
		{
			"batter":
				[
					{ "id": "1001", "type": "Regular" },
					{ "id": "1002", "type": "Chocolate" },
					{ "id": "1003", "type": "Blueberry" },
					{ "id": "1004", "type": "Devil's Food" }
				]
		},
	"toppings":
		{
			"topping":
			[
				{ "id": "5001", "type": "None" },
				{ "id": "5002", "type": "Glazed" },
				{ "id": "5005", "type": "Sugar" },
				{ "id": "5007", "type": "Powdered Sugar" },
				{ "id": "5006", "type": "Chocolate with Sprinkles" },
				{ "id": "5003", "type": "Chocolate" },
				{ "id": "5004", "type": "Maple" }
			]
		}
}"#;
        static ref TOKENIZED_JSON: &'static str = "ObjectOpen({) -> NewLine -> Whitespace(\t) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"0001\",0001) -> PropertyDelimiter(,) -> NewLine -> Whitespace(\t) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"donut\",donut) -> PropertyDelimiter(,) -> NewLine -> Whitespace(\t) -> PropertyName(\"name\",name) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Cake\",Cake) -> PropertyDelimiter(,) -> NewLine -> Whitespace(\t) -> PropertyName(\"ppu\",ppu) -> KeyValueDelimiter(:) -> Whitespace( ) -> FloatValue(0.55,0.55) -> PropertyDelimiter(,) -> NewLine -> Whitespace(\t) -> PropertyName(\"style\",style) -> KeyValueDelimiter(:) -> Whitespace( ) -> ArrayOpen([) -> Whitespace( ) -> StringValue(\"hole\",hole) -> ArrayItemDelimiter(,) -> Whitespace( ) -> StringValue(\"filled\",filled) -> Whitespace( ) -> ArrayClose(]) -> PropertyDelimiter(,) -> NewLine -> Whitespace(\t) -> PropertyName(\"batters\",batters) -> KeyValueDelimiter(:) -> NewLine -> Whitespace(\t\t) -> ObjectOpen({) -> NewLine -> Whitespace(\t\t\t) -> PropertyName(\"batter\",batter) -> KeyValueDelimiter(:) -> NewLine -> Whitespace(\t\t\t\t) -> ArrayOpen([) -> NewLine -> Whitespace(\t\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"1001\",1001) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Regular\",Regular) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"1002\",1002) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Chocolate\",Chocolate) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"1003\",1003) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Blueberry\",Blueberry) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"1004\",1004) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Devil's Food\",Devil's Food) -> Whitespace( ) -> ObjectClose(}) -> NewLine -> Whitespace(\t\t\t\t) -> ArrayClose(]) -> NewLine -> Whitespace(\t\t) -> ObjectClose(}) -> PropertyDelimiter(,) -> NewLine -> Whitespace(\t) -> PropertyName(\"toppings\",toppings) -> KeyValueDelimiter(:) -> NewLine -> Whitespace(\t\t) -> ObjectOpen({) -> NewLine -> Whitespace(\t\t\t) -> PropertyName(\"topping\",topping) -> KeyValueDelimiter(:) -> NewLine -> Whitespace(\t\t\t) -> ArrayOpen([) -> NewLine -> Whitespace(\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"5001\",5001) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"None\",None) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"5002\",5002) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Glazed\",Glazed) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"5005\",5005) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Sugar\",Sugar) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"5007\",5007) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Powdered Sugar\",Powdered Sugar) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"5006\",5006) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Chocolate with Sprinkles\",Chocolate with Sprinkles) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"5003\",5003) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Chocolate\",Chocolate) -> Whitespace( ) -> ObjectClose(}) -> ArrayItemDelimiter(,) -> NewLine -> Whitespace(\t\t\t\t) -> ObjectOpen({) -> Whitespace( ) -> PropertyName(\"id\",id) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"5004\",5004) -> PropertyDelimiter(,) -> Whitespace( ) -> PropertyName(\"type\",type) -> KeyValueDelimiter(:) -> Whitespace( ) -> StringValue(\"Maple\",Maple) -> Whitespace( ) -> ObjectClose(}) -> NewLine -> Whitespace(\t\t\t) -> ArrayClose(]) -> NewLine -> Whitespace(\t\t) -> ObjectClose(}) -> NewLine -> ObjectClose(})";
    }

    fn write_tokens(is_first: bool, json_lexer: &mut JsonStreamLexer, tokenized: &mut String) -> bool {
        let mut is_first = is_first;

        loop {
            match json_lexer.pop_token() {
                JsonStreamStatus::None => break,
                JsonStreamStatus::Token(token) => {
                    if is_first {
                        is_first = false;
                    } else {
                        tokenized.push_str(format!(" -> ").as_str());
                    }
                    tokenized.push_str(format!("{}", token).as_str());
                    write_token(tokenized, token);
                }
            }
        }

        is_first
    }

    fn write_token(tokenized: &mut String, token: JsonToken) {
        match token {
            JsonToken::PropertyName { raw, name } => {
                tokenized.push_str(format!("({},{})", raw, name).as_str());
            }
            JsonToken::BooleanValue { raw, value } => {
                tokenized.push_str(format!("({},{})", raw, value).as_str());
            }
            JsonToken::StringValue { raw, value } => {
                tokenized.push_str(format!("({},{})", raw, value).as_str());
            }
            JsonToken::IntegerValue { raw, value } => {
                tokenized.push_str(format!("({},{})", raw, value).as_str());
            }
            JsonToken::FloatValue { raw, value } => {
                tokenized.push_str(format!("({},{})", raw, value).as_str());
            }
            JsonToken::ObjectOpen(raw) => {
                tokenized.push_str(format!("({})", raw).as_str());
            }
            JsonToken::ObjectClose(raw) => {
                tokenized.push_str(format!("({})", raw).as_str());
            }
            JsonToken::ArrayOpen(raw) => {
                tokenized.push_str(format!("({})", raw).as_str());
            }
            JsonToken::ArrayClose(raw) => {
                tokenized.push_str(format!("({})", raw).as_str());
            }
            JsonToken::Whitespace(whitespace) => {
                tokenized.push_str(format!("({})", whitespace).as_str());
            }
            JsonToken::NewLine(_) => {
                tokenized.push_str(format!("").as_str());
            }
            JsonToken::ArrayItemDelimiter(delimiter) => {
                tokenized.push_str(format!("({})", delimiter).as_str());
            }
            JsonToken::PropertyDelimiter(delimiter) => {
                tokenized.push_str(format!("({})", delimiter).as_str());
            }
            JsonToken::KeyValueDelimiter(delimiter) => {
                tokenized.push_str(format!("({})", delimiter).as_str());
            }
        }
    }

    #[test]
    fn test_lexer() {
        let mut json_lexer = JsonStreamLexer::new();

        let mut is_first = true;

        let mut tokenized = String::new();

        for c in TABBED_JSON_SAMPLE.chars() {
            if let Ok(_) = json_lexer.push_char(c) {
                assert!(true);
            } else {
                assert!(false);
            }

            is_first = write_tokens(is_first, &mut json_lexer, &mut tokenized);
        }

        json_lexer.close();

        assert_eq!(tokenized, String::from(TOKENIZED_JSON.clone()));
    }
}
