use std::collections::VecDeque;

use strum_macros::Display;

#[derive(PartialEq)]
enum StructType {
    Object,
    Array,
}

#[derive(PartialEq)]
enum ReadMode {
    String,
    Number,
    Whitespace,
    None,
}

#[derive(Display, PartialEq)]
pub enum JsonTokenType {
    PropertyName,
    StringValue,
    NumberValue,
    ObjectOpen,
    ObjectClose,
    ArrayOpen,
    ArrayClose,
    Whitespace,
    NewLine,
    PropertyDelimiter,
    KeyValueDelimiter,
}

pub struct JsonStreamToken {
    pub token_raw: String,
    pub token_parsed: String,
    pub token_type: JsonTokenType,
}

pub enum JsonStreamStatus {
    None,
    Token(JsonStreamToken),
    Finish,
}

pub struct JsonStreamLexer {
    struct_type_stack: Vec<StructType>,
    property_name_toggle: bool,
    read_mode: ReadMode,
    raw_token_builder: String,
    parsed_token_builder: String,
    parsed_tokens: VecDeque<JsonStreamStatus>,
}

impl JsonStreamLexer {
    pub fn new() -> JsonStreamLexer {
        JsonStreamLexer {
            struct_type_stack: Vec::new(),
            read_mode: ReadMode::None,
            property_name_toggle: true,
            raw_token_builder: String::new(),
            parsed_token_builder: String::new(),
            parsed_tokens: VecDeque::new(),
        }
    }

    fn is_in_array(&self) -> bool {
        match self.struct_type_stack.last() {
            Some(struct_type) => match struct_type {
                StructType::Object => false,
                StructType::Array => true,
            },
            None => panic!("no opening body"),
        }
    }

    pub fn pop_token(&mut self) -> JsonStreamStatus {
        match self.parsed_tokens.pop_front() {
            Some(status) => status,
            None => JsonStreamStatus::None,
        }
    }

    pub fn push_char(&mut self, c: char) {
        match c {
            '{' => {
                let object_open_token = JsonStreamToken {
                    token_raw: String::from("{"),
                    token_parsed: String::from("{"),
                    token_type: JsonTokenType::ObjectOpen,
                };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => panic!("unexpected character when reading a number"),
                    ReadMode::Whitespace => {
                        self.struct_type_stack.push(StructType::Object);

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::Whitespace,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(object_open_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.property_name_toggle = true;
                    }
                    ReadMode::None => {
                        self.struct_type_stack.push(StructType::Object);

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(object_open_token));

                        self.property_name_toggle = true;
                    }
                }
            }
            '}' => {
                let object_close_token = JsonStreamToken {
                    token_raw: String::from("}"),
                    token_parsed: String::from("}"),
                    token_type: JsonTokenType::ObjectClose,
                };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => {
                        if self
                            .struct_type_stack
                            .pop()
                            .unwrap_or_else(|| panic!("stack ended unexpectedly"))
                            != StructType::Object
                        {
                            panic!("expected to close an object but closed an array");
                        }

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::NumberValue,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(object_close_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::Whitespace => {
                        if self
                            .struct_type_stack
                            .pop()
                            .unwrap_or_else(|| panic!("stack ended unexpectedly"))
                            != StructType::Object
                        {
                            panic!("expected to close an object but closed an array");
                        }

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::Whitespace,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(object_close_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::None => {
                        if self
                            .struct_type_stack
                            .pop()
                            .unwrap_or_else(|| panic!("stack ended unexpectedly"))
                            != StructType::Object
                        {
                            panic!("expected to close an object but closed an array");
                        }

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(object_close_token));
                    }
                }
            }
            '[' => {
                let array_open_token = JsonStreamToken {
                    token_raw: String::from("["),
                    token_parsed: String::from("["),
                    token_type: JsonTokenType::ArrayOpen,
                };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => panic!("unexpected character when reading a number"),
                    ReadMode::Whitespace => {
                        self.struct_type_stack.push(StructType::Array);

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::Whitespace,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(array_open_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::None => {
                        self.struct_type_stack.push(StructType::Array);

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(array_open_token));
                    }
                }
            }
            ']' => {
                let array_close_token = JsonStreamToken {
                    token_raw: String::from("]"),
                    token_parsed: String::from("]"),
                    token_type: JsonTokenType::ArrayClose,
                };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => {
                        if self
                            .struct_type_stack
                            .pop()
                            .unwrap_or_else(|| panic!("stack ended unexpectedly"))
                            != StructType::Array
                        {
                            panic!("expected to close an array but closed an object");
                        }

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::NumberValue,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(array_close_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::Whitespace => {
                        if self
                            .struct_type_stack
                            .pop()
                            .unwrap_or_else(|| panic!("stack ended unexpectedly"))
                            != StructType::Array
                        {
                            panic!("expected to close an array but closed an object");
                        }

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::Whitespace,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(array_close_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::None => {
                        if self
                            .struct_type_stack
                            .pop()
                            .unwrap_or_else(|| panic!("stack ended unexpectedly"))
                            != StructType::Array
                        {
                            panic!("expected to close an array but closed an object");
                        }

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(array_close_token));
                    }
                }
            }
            '"' => {
                if self.read_mode == ReadMode::Whitespace {
                    self.parsed_tokens
                        .push_back(JsonStreamStatus::Token(JsonStreamToken {
                            token_raw: self.raw_token_builder.clone(),
                            token_parsed: self.parsed_token_builder.clone(),
                            token_type: JsonTokenType::Whitespace,
                        }));
                    self.raw_token_builder.clear();
                    self.parsed_token_builder.clear();
                }

                let array_toggle = self.is_in_array();

                self.raw_token_builder.push('"');

                match self.read_mode {
                    ReadMode::String => {
                        let token_type = if self.property_name_toggle && !array_toggle {
                            JsonTokenType::PropertyName
                        } else {
                            JsonTokenType::StringValue
                        };

                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type,
                            }));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::None;
                    }
                    ReadMode::Number => {
                        panic!("malformed JSON, reading a number didn't expect a \"")
                    }
                    _ => {
                        self.read_mode = ReadMode::String;
                    }
                }
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                match self.read_mode {
                    ReadMode::Whitespace => {
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::Whitespace,
                            }));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::Number
                    }
                    ReadMode::None => self.read_mode = ReadMode::Number,
                    _ => {}
                }

                self.raw_token_builder.push(c);
                self.parsed_token_builder.push(c);
            }
            ',' => {
                let delimiter_token = JsonStreamToken {
                    token_raw: String::from(","),
                    token_parsed: String::from(","),
                    token_type: JsonTokenType::PropertyDelimiter,
                };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => {
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::NumberValue,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(delimiter_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::None;

                        self.property_name_toggle = match self.struct_type_stack.last() {
                            Some(struct_type) => match struct_type {
                                StructType::Object => true,
                                StructType::Array => false,
                            },
                            None => panic!("no opening body"),
                        };
                    }
                    ReadMode::Whitespace => {
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::Whitespace,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(delimiter_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::None;

                        self.property_name_toggle = match self.struct_type_stack.last() {
                            Some(struct_type) => match struct_type {
                                StructType::Object => true,
                                StructType::Array => false,
                            },
                            None => panic!("no opening body"),
                        };
                    }
                    ReadMode::None => {
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(delimiter_token));

                        self.property_name_toggle = match self.struct_type_stack.last() {
                            Some(struct_type) => match struct_type {
                                StructType::Object => true,
                                StructType::Array => false,
                            },
                            None => panic!("no opening body"),
                        };
                    }
                }
            }
            ' ' | '\t' => {
                match self.read_mode {
                    ReadMode::Number => {
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::NumberValue,
                            }));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::Whitespace;
                    }
                    ReadMode::None => {
                        self.read_mode = ReadMode::Whitespace;
                    }
                    _ => {}
                }

                self.raw_token_builder.push(c);
                self.parsed_token_builder.push(c);
            }
            ':' => {
                self.property_name_toggle = !self.property_name_toggle;

                self.parsed_tokens
                    .push_back(JsonStreamStatus::Token(JsonStreamToken {
                        token_raw: String::from(":"),
                        token_parsed: String::from(":"),
                        token_type: JsonTokenType::KeyValueDelimiter,
                    }));
            }
            '\n' => {
                let newline_token = JsonStreamToken {
                    token_raw: String::from("\n"),
                    token_parsed: String::from("\n"),
                    token_type: JsonTokenType::NewLine,
                };

                match self.read_mode {
                    ReadMode::String => panic!("unexpected end of string"),
                    ReadMode::Number => {
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::NumberValue,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(newline_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::None;
                    }
                    ReadMode::Whitespace => {
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(JsonStreamToken {
                                token_raw: self.raw_token_builder.clone(),
                                token_parsed: self.parsed_token_builder.clone(),
                                token_type: JsonTokenType::Whitespace,
                            }));
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(newline_token));
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::None;
                    }
                    ReadMode::None => {
                        self.parsed_tokens
                            .push_back(JsonStreamStatus::Token(newline_token));
                    }
                }
            }
            _ => {
                self.raw_token_builder.push(c);
                self.parsed_token_builder.push(c);
            }
        }
    }
}
