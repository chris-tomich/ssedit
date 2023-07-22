mod json;

use std::{io::{self, BufRead}, collections::LinkedList};
use json::lexer::{JsonStreamToken, JsonStreamLexer, JsonStream, JsonTokenType};

fn main() -> io::Result<()> {
    let read = false;

    if read {
        analyse();
    }
    else {
        find();
    }

    Ok(())
}

fn find() {
    let mut json_getter = JsonGet::new("batters");
    let mut lines = io::stdin().lock().lines();

    let mut json_lexer = JsonStreamLexer::new();

    let mut eof = false;

    while !eof {
        match lines.next() {
            Some(result) => {
                let line = result.unwrap_or_else(|_|{eof = true; String::new()});

                for c in line.chars() {
                    let output = json_getter.parse(json_lexer.analyse(c));

                    if let Some(value) = output {
                        println!("found value '{}'", value);
                        return;
                    }
                }
            },
            None => break,
        }

        let output = json_getter.parse(json_lexer.analyse('\n'));

        if let Some(value) = output {
            println!("found value '{}'", value);
            return;
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

#[derive(PartialEq)]
enum JsonGetParseMode {
    Search,
    Capture,
}

enum JsonPathComponent {
    PropertyName(String),
    ArrayIndex(usize),
}

struct JsonGet {
    path: LinkedList<JsonPathComponent>,
    current_token: JsonPathComponent,
    parse_mode: JsonGetParseMode,
    captured_tokens: Vec<JsonStreamToken>,
}

impl JsonGet {
    fn new(path: &str) -> JsonGet {
        let mut path = process_path(path);
        if let Some(starting_token) = path.pop_front() {
            JsonGet { path: path, current_token: starting_token, parse_mode: JsonGetParseMode::Search, captured_tokens: Vec::new() }
        } else {
            panic!("path isn't supported")
        }
    }

    fn parse(&mut self, stream: JsonStream) -> Option<String> {
        match stream {
            JsonStream::None => {}
            JsonStream::Single(token) => {
                if self.process_token(&token) {
                    self.captured_tokens.push(token);
                }
            }
            JsonStream::Double(token1, token2) => {
                if self.process_token(&token1) {
                    self.captured_tokens.push(token1);
                }

                if self.process_token(&token2) {
                    self.captured_tokens.push(token2);
                }
            }
        }

        None
    }

    fn process_token(&mut self, token: &JsonStreamToken) -> bool {
        if token.token_type == JsonTokenType::PropertyName && self.parse_mode == JsonGetParseMode::Search {
            if let JsonPathComponent::PropertyName(name) = &self.current_token {
                if token.token_parsed.eq(name) {
                    if let Some(next_token) = self.path.pop_front() {
                        self.current_token = next_token;
                    } else {
                        self.parse_mode = JsonGetParseMode::Capture;
                    }
                }
            }

            false
        } else if self.parse_mode == JsonGetParseMode::Capture {
            true
        } else {
            false
        }
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

    parsed_path.push_back(JsonPathComponent::PropertyName(token_builder.clone()));

    parsed_path
}