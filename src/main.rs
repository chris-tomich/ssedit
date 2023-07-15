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

                for c in line.chars() {
                    match json_lexer.analyse(c) {
                        JsonStream::None => {}
                        JsonStream::Single(token) => tokens.push(token),
                        JsonStream::Double(token1, token2) => {
                            tokens.push(token1);
                            tokens.push(token2);
                        }
                    }
                }
            },
            None => break,
        }

        match json_lexer.analyse('\n') {
            JsonStream::None => {}
            JsonStream::Single(token) => tokens.push(token),
            JsonStream::Double(token1, token2) => {
                tokens.push(token1);
                tokens.push(token2);
            }
        }
    }

    let mut raw_input = String::new();

    for token in &tokens {
        match token.token_type {
            JsonTokenType::NewLine => println!("{}", token.token_type),
            _ => println!("{}: '{}'", token.token_type, token.token_parsed),
        }

        raw_input.push_str(token.token_raw.as_str());
    }

    println!("{}", raw_input);

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
    PropertyDelimiter,
    KeyValueDelimiter,
}

struct JsonStreamToken {
    token_raw: String,
    token_parsed: String,
    token_type: JsonTokenType,
}

enum JsonStream {
    None,
    Single(JsonStreamToken),
    Double(JsonStreamToken, JsonStreamToken),
}

struct JsonStreamLexer {
    struct_type_stack: Vec<StructType>,
    property_name_toggle: bool,
    read_mode: ReadMode,
    raw_token_builder: String,
    parsed_token_builder: String,
}

impl JsonStreamLexer {
    fn new() -> JsonStreamLexer {
        JsonStreamLexer { struct_type_stack: Vec::new(), read_mode: ReadMode::None, property_name_toggle: true, raw_token_builder: String::new(), parsed_token_builder: String::new() }
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

    fn analyse(&mut self, c: char) -> JsonStream {
        match c {
            '{' => {
                let mut token = JsonStream::None;
                let object_open_token = JsonStreamToken { token_raw: String::from("{"), token_parsed: String::from("{"), token_type: JsonTokenType::ObjectOpen };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => panic!("unexpected character when reading a number"),
                    ReadMode::Whitespace => {
                        self.struct_type_stack.push(StructType::Object);

                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::Whitespace }, object_open_token);
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::None => {
                        self.struct_type_stack.push(StructType::Object);

                        token = JsonStream::Single(object_open_token);
                    }
                }
                
                token
            }
            '}' => {
                let mut token = JsonStream::None;
                let object_close_token = JsonStreamToken { token_raw: String::from("}"), token_parsed: String::from("}"), token_type: JsonTokenType::ObjectClose };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => {
                        if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Object {
                            panic!("expected to close an object but closed an array");
                        }

                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::NumberValue }, object_close_token);
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::Whitespace => {
                        if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Object {
                            panic!("expected to close an object but closed an array");
                        }

                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::Whitespace }, object_close_token);
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::None => {
                        if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Object {
                            panic!("expected to close an object but closed an array");
                        }

                        token = JsonStream::Single(object_close_token);
                    }
                }

                token
            }
            '[' => {
                let mut token = JsonStream::None;
                let array_open_token = JsonStreamToken { token_raw: String::from("["), token_parsed: String::from("["), token_type: JsonTokenType::ArrayOpen };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => panic!("unexpected character when reading a number"),
                    ReadMode::Whitespace => {
                        self.struct_type_stack.push(StructType::Array);

                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::Whitespace }, array_open_token);
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::None => {
                        self.struct_type_stack.push(StructType::Array);

                        token = JsonStream::Single(array_open_token);
                    }
                }

                token
            }
            ']' => {
                let mut token = JsonStream::None;
                let array_close_token = JsonStreamToken { token_raw: String::from("]"), token_parsed: String::from("]"), token_type: JsonTokenType::ArrayClose };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => {
                        if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Array {
                            panic!("expected to close an array but closed an object");
                        }

                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::NumberValue }, array_close_token);
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::Whitespace => {
                        if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Array {
                            panic!("expected to close an array but closed an object");
                        }

                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::Whitespace }, array_close_token);
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                    }
                    ReadMode::None => {
                        if self.struct_type_stack.pop().unwrap_or_else(||panic!("stack ended unexpectedly")) != StructType::Array {
                            panic!("expected to close an array but closed an object");
                        }

                        token = JsonStream::Single(array_close_token);
                    }
                }

                token
            }
            '"' => {
                let mut token = JsonStream::None;

                if self.read_mode == ReadMode::Whitespace {
                    token = JsonStream::Single(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::Whitespace });
                    self.raw_token_builder.clear();
                    self.parsed_token_builder.clear();
                }

                let array_toggle = self.is_in_array();

                self.raw_token_builder.push('"');

                match self.read_mode {
                    ReadMode::String => {
                        let token_type = if self.property_name_toggle && !array_toggle {
                            JsonTokenType::PropertyName
                        }
                        else {
                            JsonTokenType::StringValue
                        };
                        
                        token = JsonStream::Single(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type });
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::None;
                    }
                    ReadMode::Number => panic!("malformed JSON, reading a number didn't expect a \""),
                    _ => {
                        self.read_mode = ReadMode::String;
                    }
                }

                token
            }
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                let mut token = JsonStream::None;

                match self.read_mode {
                    ReadMode::Whitespace => {
                        token = JsonStream::Single(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::Whitespace });
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::Number
                    }
                    ReadMode::None => {
                        self.read_mode = ReadMode::Number
                    }
                    _ => {}
                }

                self.raw_token_builder.push(c);
                self.parsed_token_builder.push(c);

                token
            }
            ',' => {
                let mut token = JsonStream::None;
                let delimiter_token = JsonStreamToken { token_raw: String::from(","), token_parsed: String::from(","), token_type: JsonTokenType::PropertyDelimiter };

                match self.read_mode {
                    ReadMode::String => {
                        self.raw_token_builder.push(c);
                        self.parsed_token_builder.push(c);
                    }
                    ReadMode::Number => {
                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::NumberValue }, delimiter_token);
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
                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::Whitespace }, delimiter_token);
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
                        token = JsonStream::Single(delimiter_token);

                        self.property_name_toggle = match self.struct_type_stack.last() {
                            Some(struct_type) => match struct_type {
                                StructType::Object => true,
                                StructType::Array => false,
                            },
                            None => panic!("no opening body"),
                        };
                    }
                }

                token
            }
            ' ' | '\t' => {
                let mut token = JsonStream::None;

                match self.read_mode {
                    ReadMode::Number => {
                        token = JsonStream::Single(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::NumberValue });
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

                token
            }
            ':' => {
                self.property_name_toggle = !self.property_name_toggle;

                JsonStream::Single(JsonStreamToken { token_raw: String::from(":"), token_parsed: String::from(":"), token_type: JsonTokenType::KeyValueDelimiter })
            }
            '\n' => {
                let token;
                let newline_token = JsonStreamToken { token_raw: String::from("\n"), token_parsed: String::from("\n"), token_type: JsonTokenType::NewLine };

                match self.read_mode {
                    ReadMode::String => panic!("unexpected end of string"),
                    ReadMode::Number => {
                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::NumberValue }, newline_token);
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();
                        
                        self.read_mode = ReadMode::None;
                    }
                    ReadMode::Whitespace => {
                        token = JsonStream::Double(JsonStreamToken { token_raw: self.raw_token_builder.clone(), token_parsed: self.parsed_token_builder.clone(), token_type: JsonTokenType::Whitespace }, newline_token);
                        self.raw_token_builder.clear();
                        self.parsed_token_builder.clear();

                        self.read_mode = ReadMode::None;
                    }
                    ReadMode::None => {
                        token = JsonStream::Single(newline_token);
                    }
                }

                token
            }
            _ => {
                self.raw_token_builder.push(c);
                self.parsed_token_builder.push(c);
                
                JsonStream::None
            }
        }
    }
}