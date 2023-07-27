mod json;

use std::{io::{self, BufRead}, collections::LinkedList, env};
use json::lexer::{JsonStreamToken, JsonStreamLexer, JsonStream, JsonTokenType};
use strum_macros::Display;

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
    let args: Vec<_> = env::args().collect();
    
    let search_path = if args.len() >= 2 {
        args[1].as_str()
    } else {
        "batters.batter[2]"
    };

    let mut json_getter = JsonGet::new(search_path);
    let mut lines = io::stdin().lock().lines();

    let mut json_lexer = JsonStreamLexer::new();

    let mut eof = false;

    let mut capture_mode = false;

    print!("");

    while !eof {
        match lines.next() {
            Some(result) => {
                let line = result.unwrap_or_else(|_|{eof = true; String::new()});

                for c in line.chars() {
                    match json_getter.parse(json_lexer.analyse(c)) {
                        JsonStream::None => {}
                        JsonStream::Single(token) => {
                            if !capture_mode {
                                capture_mode = true;
                                print!("|");
                            }
                            print!("{}", token.token_raw);
                        }
                        JsonStream::Double(token1, token2) => {
                            if !capture_mode {
                                capture_mode = true;
                                print!("|");
                            }
                            print!("{}{}", token1.token_raw, token2.token_raw);
                        }
                        JsonStream::Finish => {
                            if capture_mode {
                                print!("|");
                                capture_mode = false;
                            }
                        }
                    }

                    if !capture_mode {
                        print!("{}", c);
                    }
                }

                print!("\n");
            },
            None => break,
        }

        match json_getter.parse(json_lexer.analyse('\n')) {
            JsonStream::None => {}
            JsonStream::Single(token) => print!("{}", token.token_raw),
            JsonStream::Double(token1, token2) => print!("{}{}", token1.token_raw, token2.token_raw),
            JsonStream::Finish => {}
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
                        JsonStream::Finish => {}
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
            JsonStream::Finish => {}
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
    Finish,
}

#[derive(Display)]
enum JsonPathComponent {
    PropertyName(String),
    ArrayIndex(isize),
}

struct JsonGet {
    path: LinkedList<JsonPathComponent>,
    current_token: JsonPathComponent,
    parse_mode: JsonGetParseMode,
    current_search_depth: isize,
    current_capture_depth: isize,
    current_index: isize,
}

impl JsonGet {
    fn new(path: &str) -> JsonGet {
        let mut path = process_path(path);
        if let Some(starting_token) = path.pop_front() {
            JsonGet { path: path, current_token: starting_token, parse_mode: JsonGetParseMode::Search, current_search_depth: -1, current_capture_depth: 0, current_index: -1 }
        } else {
            panic!("path isn't supported")
        }
    }

    fn parse(&mut self, stream: JsonStream) -> JsonStream {
        match stream {
            JsonStream::None => {}
            JsonStream::Single(token) => {
                if self.process_token(&token) {
                    return JsonStream::Single(token);
                } else if self.parse_mode == JsonGetParseMode::Finish {
                    return JsonStream::Finish;
                }
            }
            JsonStream::Double(token1, token2) => {
                let mut token1_processed = false;

                if self.process_token(&token1) {
                    token1_processed = true;
                } else if self.parse_mode == JsonGetParseMode::Finish {
                    return JsonStream::Finish;
                }

                if self.process_token(&token2) {
                    if token1_processed {
                        return JsonStream::Double(token1, token2);
                    } else {
                        return JsonStream::Single(token2);
                    }
                } else if self.parse_mode == JsonGetParseMode::Finish {
                    if token1_processed {
                        return JsonStream::Single(token1);
                    }

                    return JsonStream::Finish;
                }

                if token1_processed {
                    return JsonStream::Single(token1);
                }
            }
            JsonStream::Finish => return stream,
        }

        JsonStream::None
    }

    fn process_token(&mut self, token: &JsonStreamToken) -> bool {
        match self.parse_mode {
            JsonGetParseMode::Search => {
                match &self.current_token {
                    JsonPathComponent::PropertyName(name) => {
                        match token.token_type {
                            JsonTokenType::PropertyName => {
                                if token.token_parsed.eq(name) {
                                    if let Some(next_token) = self.path.pop_front() {
                                        self.current_token = next_token;
                                    } else {
                                        self.parse_mode = JsonGetParseMode::Capture;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    JsonPathComponent::ArrayIndex(index) => {
                        match token.token_type {
                            JsonTokenType::ObjectOpen => self.current_search_depth += 1,
                            JsonTokenType::ObjectClose => self.current_search_depth -= 1,
                            JsonTokenType::ArrayOpen => {
                                if self.current_search_depth == -1 {
                                    self.current_index = 0;
                                    self.current_search_depth = 0;
                                } else {
                                    self.current_search_depth += 1;
                                }
                            }
                            JsonTokenType::ArrayClose => {
                                if self.current_search_depth == 0 {
                                    panic!("index not found");
                                }
                            }
                            JsonTokenType::PropertyDelimiter => {
                                if self.current_search_depth == 0 {
                                    self.current_index += 1;
                                }
                            }
                            _ => {}
                        }

                        if self.current_index == *index && self.current_search_depth == 0 {
                            self.current_index = -1;
                            self.current_search_depth = -1;

                            if let Some(next_token) = self.path.pop_front() {
                                self.current_token = next_token;
                            } else {
                                self.parse_mode = JsonGetParseMode::Capture;
                            }
                        }
                    }
                }
                
                false
            }
            JsonGetParseMode::Capture => {
                match token.token_type {
                    JsonTokenType::ObjectOpen => self.current_capture_depth += 1,
                    JsonTokenType::ObjectClose => {
                        self.current_capture_depth -= 1;

                        if self.current_capture_depth == 0 {
                            self.parse_mode = JsonGetParseMode::Finish;
                        } else if self.current_capture_depth == -1 {
                            self.parse_mode = JsonGetParseMode::Finish;
                            return false
                        }
                    }
                    JsonTokenType::ArrayOpen => self.current_capture_depth += 1,
                    JsonTokenType::ArrayClose => {
                        self.current_capture_depth -= 1;

                        if self.current_capture_depth == 0 {
                            self.parse_mode = JsonGetParseMode::Finish;
                        } else if self.current_capture_depth == -1 {
                            self.parse_mode = JsonGetParseMode::Finish;
                            return false
                        }
                    }
                    JsonTokenType::KeyValueDelimiter => {
                        if self.current_capture_depth == 0 {
                            return false;
                        }
                    }
                    JsonTokenType::PropertyDelimiter => {
                        if self.current_capture_depth == 0 {
                            self.parse_mode = JsonGetParseMode::Finish;
                            return false;
                        }
                    }
                    _ => {}
                }

                true
            }
            JsonGetParseMode::Finish => false,
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
                if token_builder.len() > 0 {
                    parsed_path.push_back(JsonPathComponent::PropertyName(token_builder.clone()));
                    token_builder.clear();
                }
            }
            '[' => {
                parsed_path.push_back(JsonPathComponent::PropertyName(token_builder.clone()));
                token_builder.clear();

                parsing_array_index = true;
            }
            ']' => {
                if !parsing_array_index {
                    panic!("unexpected array index close");
                }

                if let Ok(array_index) = token_builder.as_str().parse::<isize>() {
                    parsed_path.push_back(JsonPathComponent::ArrayIndex(array_index));
                    token_builder.clear();
                } else {
                    panic!("array index couldn't be parsed to a number");
                }
            }
            _ => token_builder.push(c),
        }
    }
    
    if token_builder.len() > 0 {
        parsed_path.push_back(JsonPathComponent::PropertyName(token_builder.clone()));
    }

    parsed_path
}