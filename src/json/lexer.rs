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
    ArrayItemDelimiter(String),
    PropertyDelimiter(String),
    KeyValueDelimiter(String),
}

pub enum JsonPartialToken {
    Array,
    Object,
    PropertyName,
    PropertyValue,
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
}

impl JsonStreamLexer {
    pub fn new() -> JsonStreamLexer {
        let mut partial_tokens = Vec::new();
        partial_tokens.push(JsonPartialToken::Root);

        JsonStreamLexer {
            tokens: VecDeque::new(),
            partial_tokens,
        }
    }

    pub fn close(&mut self) {
        while let Some(partial_token) = self.partial_tokens.pop() {
            match partial_token {
                JsonPartialToken::Array => {}
                JsonPartialToken::Object => {}
                JsonPartialToken::PropertyName => {}
                JsonPartialToken::PropertyValue => {}
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

    pub fn push_char(&mut self, c: char) {
        match c {
            '{' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => {
                            self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Array);
                            self.partial_tokens.push(JsonPartialToken::Object);
                        }
                        JsonPartialToken::Object => todo!(),
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => {
                            self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Object);
                        }
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
                        JsonPartialToken::NumberValue(_) => todo!(),
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                            self.tokens.push_back(JsonToken::ObjectOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Object);
                            self.partial_tokens.push(JsonPartialToken::PropertyName);
                        }
                    }
                } else {
                    todo!();
                }
            }
            '}' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => todo!(),
                        JsonPartialToken::Object => self.tokens.push_back(JsonToken::ObjectClose(String::from(c))),
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => self.tokens.push_back(JsonToken::ObjectClose(String::from(c))),
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => todo!(),
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
                                    _ => todo!(),
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
                                                _ => todo!(),
                                            }
                                        }
                                    }
                                    _ => todo!(),
                                }
                            }
                        }
                    }
                } else {
                    todo!();
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
                        JsonPartialToken::Object => todo!(),
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => {
                            self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                            self.partial_tokens.push(JsonPartialToken::Array);
                        }
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => {
                            self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::Root);
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                        }
                        JsonPartialToken::NumberValue(_) => todo!(),
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {
                                        self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
                                    JsonPartialToken::Object => todo!(),
                                    JsonPartialToken::PropertyName => todo!(),
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
                                    JsonPartialToken::StringValue { raw: _, value: _ } => todo!(),
                                    JsonPartialToken::Root => {
                                        self.tokens.push_back(JsonToken::ArrayOpen(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::Root);
                                        self.partial_tokens.push(JsonPartialToken::Array);
                                    }
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
            ']' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.tokens.push_back(JsonToken::ArrayClose(String::from(c))),
                        JsonPartialToken::Object => todo!(),
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => todo!(),
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
                                    _ => todo!(),
                                }
                            }
                        }
                        JsonPartialToken::Whitespace(whitespace) => {
                            self.tokens.push_back(JsonToken::Whitespace(whitespace));
                            self.tokens.push_back(JsonToken::ArrayClose(String::from(c)));

                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => {}
                                    _ => todo!(),
                                }
                            }
                        }
                    }
                } else {
                    todo!();
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
                        JsonPartialToken::Object => todo!(),
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
                                    JsonPartialToken::StringValue { raw: _, value: _ } => todo!(),
                                    JsonPartialToken::Root => todo!(),
                                    JsonPartialToken::NumberValue(_) => todo!(),
                                    JsonPartialToken::Whitespace(_) => todo!(),
                                }
                            } else {
                                todo!();
                            }
                        }
                        JsonPartialToken::Root => todo!(),
                        JsonPartialToken::NumberValue(_) => todo!(),
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
                                    JsonPartialToken::StringValue { raw: _, value: _ } => todo!(),
                                    JsonPartialToken::Root => todo!(),
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
                                    JsonPartialToken::PropertyName => todo!(),
                                    JsonPartialToken::PropertyValue => {
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                        self.partial_tokens.push(JsonPartialToken::Whitespace(String::from(c)));
                                    }
                                    JsonPartialToken::StringValue { raw: _, value: _ } => todo!(),
                                    JsonPartialToken::Root => todo!(),
                                    JsonPartialToken::NumberValue(_) => todo!(),
                                    JsonPartialToken::Whitespace(_) => todo!(),
                                }
                            } else {
                                todo!();
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
                        JsonPartialToken::Array => todo!(),
                        JsonPartialToken::Object => todo!(),
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => {
                            self.tokens.push_back(JsonToken::KeyValueDelimiter(String::from(c)));
                            self.partial_tokens.push(JsonPartialToken::PropertyValue);
                        }
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => todo!(),
                        JsonPartialToken::NumberValue(_) => todo!(),
                        JsonPartialToken::Whitespace(whitespace) => {
                            if let Some(partial_token) = self.partial_tokens.pop() {
                                match partial_token {
                                    JsonPartialToken::Array => todo!(),
                                    JsonPartialToken::Object => todo!(),
                                    JsonPartialToken::PropertyName => todo!(),
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.tokens.push_back(JsonToken::KeyValueDelimiter(String::from(c)));
                                        self.partial_tokens.push(JsonPartialToken::PropertyValue);
                                    }
                                    JsonPartialToken::StringValue { raw: _, value: _ } => todo!(),
                                    JsonPartialToken::Root => todo!(),
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
                        JsonPartialToken::StringValue { raw: _, value: _ } => panic!("malformed string"),
                        JsonPartialToken::Root => todo!(),
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
                                    JsonPartialToken::StringValue { raw: _, value: _ } => todo!(),
                                    JsonPartialToken::Root => todo!(),
                                    JsonPartialToken::NumberValue(_) => todo!(),
                                    JsonPartialToken::Whitespace(_) => todo!(),
                                }
                            } else {
                                todo!();
                            }
                        }
                        JsonPartialToken::Whitespace(_) => todo!(),
                    }
                } else {
                    todo!();
                }
            }
            '\n' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => self.partial_tokens.push(JsonPartialToken::Array),
                        JsonPartialToken::Object => self.partial_tokens.push(JsonPartialToken::Object),
                        JsonPartialToken::PropertyName => self.partial_tokens.push(JsonPartialToken::PropertyName),
                        JsonPartialToken::PropertyValue => self.partial_tokens.push(JsonPartialToken::PropertyValue),
                        JsonPartialToken::StringValue { raw: _, value: _ } => todo!(),
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
                    todo!();
                }

                self.tokens.push_back(JsonToken::NewLine(String::from(c)));
            }
            '.' => {
                if let Some(partial_token) = self.partial_tokens.pop() {
                    match partial_token {
                        JsonPartialToken::Array => todo!(),
                        JsonPartialToken::Object => todo!(),
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => todo!(),
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => todo!(),
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
                        JsonPartialToken::Array => {
                            self.partial_tokens.push(JsonPartialToken::Array);
                            self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c)));
                        }
                        JsonPartialToken::Object => todo!(),
                        JsonPartialToken::PropertyName => todo!(),
                        JsonPartialToken::PropertyValue => self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c))),
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => todo!(),
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
                                    JsonPartialToken::Object => todo!(),
                                    JsonPartialToken::PropertyName => todo!(),
                                    JsonPartialToken::PropertyValue => {
                                        self.tokens.push_back(JsonToken::Whitespace(whitespace));
                                        self.partial_tokens.push(JsonPartialToken::NumberValue(String::from(c)));
                                    }
                                    JsonPartialToken::StringValue { raw: _, value: _ } => todo!(),
                                    JsonPartialToken::Root => todo!(),
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
                        JsonPartialToken::Array => todo!(),
                        JsonPartialToken::Object => todo!(),
                        JsonPartialToken::PropertyName => todo!("{}", c),
                        JsonPartialToken::PropertyValue => todo!("{}", c),
                        JsonPartialToken::StringValue { mut raw, mut value } => {
                            raw.push(c);
                            value.push(c);
                            self.partial_tokens.push(JsonPartialToken::StringValue { raw, value });
                        }
                        JsonPartialToken::Root => todo!(),
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
