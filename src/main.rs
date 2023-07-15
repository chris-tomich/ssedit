use std::io::{self, BufRead};
use strum_macros::Display;

fn main() -> io::Result<()> {
    let mut tokens: Vec<JsonStreamToken> = Vec::new();
    let mut lines = io::stdin().lock().lines();

    let mut json_lexer = JsonStreamLexer::new();

    let mut eof = false;

    while !eof {
        match lines.next() {
            Some(result) => {
                let line = result.unwrap_or_else(|_|{eof = true; String::new()});

                json_lexer.analyse(&mut tokens, line)
            },
            None => break,
        }
    }

    for token in tokens {
        println!("{}: '{}'", token.token_type, token.token_parsed)
    }

    Ok(())
}

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

#[derive(Display)]
enum JsonTokenType {
    PropertyName,
    StringValue,
    NumberValue,
    ObjectOpen,
    ObjectClose,
    ArrayOpen,
    ArrayClose,
    Whitespace,
    NewLine,
}

struct JsonStreamToken {
    token_raw: String,
    token_parsed: String,
    token_type: JsonTokenType,
}

struct JsonStreamLexer {
    struct_type_stack: Vec<StructType>,
    property_name_toggle: bool,
    read_mode: ReadMode,
}

impl JsonStreamLexer {
    fn new() -> JsonStreamLexer {
        JsonStreamLexer { struct_type_stack: Vec::new(), read_mode: ReadMode::None, property_name_toggle: true }
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

    fn analyse(&mut self, tokens: &mut Vec<JsonStreamToken>, line: String) {
        let mut token_builder = String::new();

        for c in line.chars() {
            match c {
                '{' => {
                    tokens.push(JsonStreamToken { token_raw: String::from("{"), token_parsed: String::from("{"), token_type: JsonTokenType::ObjectOpen });
                    self.struct_type_stack.push(StructType::Object);
                },
                '}' => {
                    tokens.push(JsonStreamToken { token_raw: String::from("}"), token_parsed: String::from("}"), token_type: JsonTokenType::ObjectClose });
                    if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Object {
                        panic!("expected to close an object but closed an array");
                    }
                },
                '[' => {
                    tokens.push(JsonStreamToken { token_raw: String::from("["), token_parsed: String::from("["), token_type: JsonTokenType::ArrayOpen });
                    self.struct_type_stack.push(StructType::Array);
                },
                ']' => {
                    tokens.push(JsonStreamToken { token_raw: String::from("]"), token_parsed: String::from("]"), token_type: JsonTokenType::ArrayClose });
                    if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Array {
                        panic!("expected to close an array but closed an object");
                    }
                },
                '"' => {
                    if self.read_mode == ReadMode::Whitespace {
                        tokens.push(JsonStreamToken { token_raw: token_builder.clone(), token_parsed: token_builder.clone(), token_type: JsonTokenType::Whitespace });
                        token_builder.clear();
                    }

                    let array_toggle = self.is_in_array();

                    token_builder.push('"');

                    match self.read_mode {
                        ReadMode::String => {
                            let token_type = if self.property_name_toggle && !array_toggle {
                                JsonTokenType::PropertyName
                            }
                            else {
                                JsonTokenType::StringValue
                            };
                            
                            tokens.push(JsonStreamToken { token_raw: token_builder.clone(), token_parsed: token_builder.clone(), token_type });
                            token_builder.clear();

                            self.read_mode = ReadMode::None;
                        }
                        ReadMode::Number => panic!("malformed JSON, reading a number didn't expect a \""),
                        _ => {
                            self.read_mode = ReadMode::String;
                        }
                    }
                },
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    match self.read_mode {
                        ReadMode::Whitespace => {
                            tokens.push(JsonStreamToken { token_raw: token_builder.clone(), token_parsed: token_builder.clone(), token_type: JsonTokenType::Whitespace });
                            token_builder.clear();

                            self.read_mode = ReadMode::Number
                        }
                        ReadMode::None => {
                            self.read_mode = ReadMode::Number
                        }
                        _ => {}
                    }

                    token_builder.push(c);
                },
                ',' => {
                    match self.read_mode {
                        ReadMode::String => {
                            token_builder.push(c);
                        }
                        ReadMode::Number => {
                            tokens.push(JsonStreamToken { token_raw: token_builder.clone(), token_parsed: token_builder.clone(), token_type: JsonTokenType::NumberValue });
                            token_builder.clear();
                            
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
                            tokens.push(JsonStreamToken { token_raw: token_builder.clone(), token_parsed: token_builder.clone(), token_type: JsonTokenType::Whitespace });
                            token_builder.clear();

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
                            self.property_name_toggle = match self.struct_type_stack.last() {
                                Some(struct_type) => match struct_type {
                                    StructType::Object => true,
                                    StructType::Array => false,
                                },
                                None => panic!("no opening body"),
                            };
                        }
                    }
                },
                ' ' => {
                    match self.read_mode {
                        ReadMode::String => token_builder.push(c),
                        ReadMode::Number => {
                            tokens.push(JsonStreamToken { token_raw: token_builder.clone(), token_parsed: token_builder.clone(), token_type: JsonTokenType::NumberValue });
                            token_builder.clear();
                            
                            self.read_mode = ReadMode::Whitespace;
                            token_builder.push(c);
                        }
                        ReadMode::Whitespace => token_builder.push(c),
                        ReadMode::None => {
                            self.read_mode = ReadMode::Whitespace;
                            token_builder.push(c);
                        }
                    }
                }
                ':' => self.property_name_toggle = !self.property_name_toggle,
                _ => token_builder.push(c),
            }
        }
    }
}