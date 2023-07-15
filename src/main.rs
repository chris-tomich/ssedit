mod json;

use std::{io::{self, BufRead}, collections::LinkedList};
use json::lexer::{JsonStreamToken, JsonStreamLexer, JsonStream, JsonTokenType};

fn main() -> io::Result<()> {
    let read = true;

    if read {
        analyse();
    }
    else {
        find();
    }

    Ok(())
}

fn find() {
    let json_getter = JsonGet::new("batters.batter[2].type");
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
}

fn analyse() {
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
}

enum JsonPathComponent {
    PropertyName(String),
    ArrayIndex(usize),
}

struct JsonGet {
    path: LinkedList<JsonPathComponent>,
    current_token: JsonPathComponent,
}

impl JsonGet {
    fn new(path: &str) -> JsonGet {
        let mut path = process_path(path);
        if let Some(starting_token) = path.pop_front() {
            JsonGet { path: path, current_token: starting_token }
        } else {
            panic!("path isn't supported")
        }
    }

    fn parse(&mut self, stream: JsonStream) -> Option<String> {
        match stream {
            JsonStream::None => {}
            JsonStream::Single(token) => {
                if token.token_type == JsonTokenType::PropertyName {
                    if let JsonPathComponent::PropertyName(name) = &self.current_token {
                        if token.token_parsed.eq(name) {
                            if let Some(next_token) = self.path.pop_front() {
                                self.current_token = next_token;
                            }
                        }
                    }
                }
            }
            JsonStream::Double(_, _) => todo!(),
        }

        None
    }
}

fn process_path(path: &str) -> LinkedList<JsonPathComponent> {
    let mut parsed_path = LinkedList::new();
    let mut token_builder = String::new();

    let mut parsing_array_index = false;

    for c in path.chars() {
        match c {
            '.' => {
                parsed_path.push_back(JsonPathComponent::PropertyName(token_builder.clone()));
                token_builder.clear();
            }
            '[' => parsing_array_index = true,
            ']' => {
                if !parsing_array_index {
                    panic!("unexpected array index close");
                }

                if let Ok(array_index) = token_builder.as_str().parse::<usize>() {
                    parsed_path.push_back(JsonPathComponent::ArrayIndex(array_index));
                } else {
                    panic!("array index couldn't be parsed to a number");
                }
            }
            _ => token_builder.push(c),
        }
    }

    parsed_path
}