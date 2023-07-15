use std::io::{self, BufRead};

const OPEN_OBJECT: &str = "open_object";
const CLOSE_OBJECT: &str = "close_object";
const OPEN_ARRAY: &str = "open_array";
const CLOSE_ARRAY: &str = "close_array";
const OPEN_TOKEN: &str = "open_token";
const CLOSE_TOKEN: &str = "close_token";

fn main() -> io::Result<()> {
    let mut tokens: Vec<String> = Vec::new();
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
        println!("{}", token)
    }

    Ok(())
}

#[derive(PartialEq)]
enum StructType {
    Object,
    Array,
}

#[derive(PartialEq)]
enum ValueType {
    String,
    Number,
    None,
}

struct JsonStreamLexer {
    struct_type_stack: Vec<StructType>,
    property_key_toggle: bool,
    value_type: ValueType,
}

impl JsonStreamLexer {
    fn new() -> JsonStreamLexer {
        JsonStreamLexer { struct_type_stack: Vec::new(), value_type: ValueType::None, property_key_toggle: true }
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

    fn analyse(&mut self, tokens: &mut Vec<String>, line: String) {
        let mut token_builder = String::new();

        for c in line.chars() {
            match c {
                '{' => {
                    tokens.push(String::from(OPEN_OBJECT));
                    self.struct_type_stack.push(StructType::Object);
                },
                '}' => {
                    tokens.push(String::from(CLOSE_OBJECT));
                    if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Object {
                        panic!("expected to close an object but closed an array");
                    }
                },
                '[' => {
                    tokens.push(String::from(OPEN_ARRAY));
                    self.struct_type_stack.push(StructType::Array);
                },
                ']' => {
                    tokens.push(String::from(CLOSE_ARRAY));
                    if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Array {
                        panic!("expected to close an array but closed an object");
                    }
                },
                '\"' => {
                    let array_toggle = self.is_in_array();

                    match self.value_type {
                        ValueType::String => {
                            if self.property_key_toggle && !array_toggle {
                                let mut property = "property_key: ".to_string();
                                property.push_str(token_builder.trim());
                                tokens.push(property);
                            }
                            else {
                                let mut property = "property_value_str: ".to_string();
                                property.push_str(token_builder.trim());
                                tokens.push(property);
                            }
                            token_builder.clear();
                            tokens.push(String::from(CLOSE_TOKEN));
                            self.value_type = ValueType::None;
                        }
                        ValueType::Number => panic!("malformed JSON, reading a number didn't expect a \""),
                        ValueType::None => {
                            tokens.push(String::from(OPEN_TOKEN));
                            self.value_type = ValueType::String;
                        }
                    }
                },
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    token_builder.push(c);
                    
                    if self.value_type != ValueType::String {
                        self.value_type = ValueType::Number;
                    }
                },
                ',' => {
                    if self.value_type == ValueType::String {
                        token_builder.push(c);
                    }
                    else {
                        if self.value_type == ValueType::Number {
                            let mut property = "property_value_num: ".to_string();
                            property.push_str(token_builder.as_str());
                            tokens.push(property);
                            
                            self.value_type = ValueType::None;
                        }

                        self.property_key_toggle = match self.struct_type_stack.last() {
                            Some(struct_type) => match struct_type {
                                StructType::Object => true,
                                StructType::Array => false,
                            },
                            None => panic!("no opening body"),
                        };
                    }
                },
                ' ' => {
                    if self.value_type == ValueType::String {
                        token_builder.push(c);
                    }
                }
                ':' => self.property_key_toggle = !self.property_key_toggle,
                _ => token_builder.push(c),
            }
        }
    }
}