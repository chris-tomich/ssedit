mod json;

use core::fmt;
use std::{io::{self, Read, BufRead}, collections::LinkedList};
use clap::Parser;
use json::lexer::{JsonStreamToken, JsonStreamLexer, JsonStream, JsonTokenType};
use strum_macros::Display;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ConfeditArgs {
    #[arg(short, long)]
    select: String,

    #[arg(short, long, default_value = "")]
    replace: String
}

fn main() -> io::Result<()> {
    let read = false;
    let finder = false;

    if read {
        analyse();
    } else if finder {
        find();
    } else {
        search();
    }

    Ok(())
}

fn find() {
    let args = ConfeditArgs::parse();
    
    let search_path = if args.select.is_empty() {
        eprintln!("no select command provided");
        return
    } else {
        args.select.as_str()
    };

    let mut json_getter = JsonSelect::new(search_path);

    let mut json_lexer = JsonStreamLexer::new();

    let mut capture_mode = false;
    let mut captured_tokens = LinkedList::new();

    let mut buffer = [0; 1];

    loop {
        match io::stdin().lock().read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let c = buffer[0] as char;

                match json_getter.parse(json_lexer.analyse(c)) {
                    JsonStream::None => {}
                    JsonStream::Single(token) => {
                        capture_mode = true;
                        captured_tokens.push_back(token);
                    }
                    JsonStream::Double(token1, token2) => {
                        capture_mode = true;
                        captured_tokens.push_back(token1);
                        captured_tokens.push_back(token2);
                    }
                    JsonStream::Finish => {
                        let mut starting = true;
                        if capture_mode {
                            for token in &captured_tokens {
                                if token.token_type != JsonTokenType::Whitespace {
                                    // We need this check because whitespace is already included in the uncapture stream.
                                    // This is hard to resolve because whitespace usually only ends once a new token has
                                    // begun meaning you don't know it's unnecessary till it's already been passed as a token.
                                    starting = false;
                                }

                                if !starting {
                                    if args.replace.is_empty() {
                                        print!("{}", token.token_parsed);
                                    } else {
                                        if token.token_type == JsonTokenType::NewLine || token.token_type == JsonTokenType::Whitespace {
                                            print!("{}", token.token_raw);
                                        } else if token.token_type == JsonTokenType::StringValue {
                                            print!("\"{}\"", args.replace);
                                            break;
                                        } else {
                                            print!("{}", args.replace);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        capture_mode = false;
                    }
                }

                if !capture_mode && !args.replace.is_empty() {
                    print!("{}", c);
                }
            }
            Err(err) => {
                eprintln!("error reading from stdin: {}", err);
                break;
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

fn search() {
    println!("Path '$.' parsed as '{}'", JsonPath::from("$."));
    println!("Path '$[10]' parsed as '{}'", JsonPath::from("$[10]"));
}

#[derive(PartialEq)]
enum JsonSelectParseMode {
    Search,
    Capture,
    Finish,
}

#[derive(Display)]
enum JsonSelectComponent {
    PropertyName(String),
    ArrayIndex(isize),
}

struct JsonSelect {
    path: LinkedList<JsonSelectComponent>,
    current_token: JsonSelectComponent,
    parse_mode: JsonSelectParseMode,
    current_search_depth: isize,
    current_capture_depth: isize,
    current_index: isize,
}

impl JsonSelect {
    fn new(path: &str) -> JsonSelect {
        let mut path = process_path(path);
        if let Some(starting_token) = path.pop_front() {
            JsonSelect { path: path, current_token: starting_token, parse_mode: JsonSelectParseMode::Search, current_search_depth: -1, current_capture_depth: 0, current_index: -1 }
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
                } else if self.parse_mode == JsonSelectParseMode::Finish {
                    return JsonStream::Finish;
                }
            }
            JsonStream::Double(token1, token2) => {
                let mut token1_processed = false;

                if self.process_token(&token1) {
                    token1_processed = true;
                } else if self.parse_mode == JsonSelectParseMode::Finish {
                    return JsonStream::Finish;
                }

                if self.process_token(&token2) {
                    if token1_processed {
                        return JsonStream::Double(token1, token2);
                    } else {
                        return JsonStream::Single(token2);
                    }
                } else if self.parse_mode == JsonSelectParseMode::Finish {
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
            JsonSelectParseMode::Search => {
                match &self.current_token {
                    JsonSelectComponent::PropertyName(name) => {
                        match token.token_type {
                            JsonTokenType::PropertyName => {
                                if token.token_parsed.eq(name) {
                                    if let Some(next_token) = self.path.pop_front() {
                                        self.current_token = next_token;
                                    } else {
                                        self.parse_mode = JsonSelectParseMode::Capture;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    JsonSelectComponent::ArrayIndex(index) => {
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
                                self.parse_mode = JsonSelectParseMode::Capture;
                            }
                        }
                    }
                }
                
                false
            }
            JsonSelectParseMode::Capture => {
                match token.token_type {
                    JsonTokenType::ObjectOpen => self.current_capture_depth += 1,
                    JsonTokenType::ObjectClose => {
                        self.current_capture_depth -= 1;

                        if self.current_capture_depth == 0 {
                            self.parse_mode = JsonSelectParseMode::Finish;
                        } else if self.current_capture_depth == -1 {
                            self.parse_mode = JsonSelectParseMode::Finish;
                            return false
                        }
                    }
                    JsonTokenType::ArrayOpen => self.current_capture_depth += 1,
                    JsonTokenType::ArrayClose => {
                        self.current_capture_depth -= 1;

                        if self.current_capture_depth == 0 {
                            self.parse_mode = JsonSelectParseMode::Finish;
                        } else if self.current_capture_depth == -1 {
                            self.parse_mode = JsonSelectParseMode::Finish;
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
                            self.parse_mode = JsonSelectParseMode::Finish;
                            return false;
                        }
                    }
                    JsonTokenType::StringValue | JsonTokenType::NumberValue => {
                        if self.current_capture_depth == 0 {
                            self.parse_mode = JsonSelectParseMode::Finish;
                            return true;
                        }
                    }
                    _ => {}
                }

                true
            }
            JsonSelectParseMode::Finish => false,
        }
    }
}

fn process_path(path: &str) -> LinkedList<JsonSelectComponent> {
    let mut parsed_path = LinkedList::new();
    let mut token_builder = String::new();

    let mut parsing_array_index = false;

    for c in path.chars() {
        match c {
            '.' => {
                if token_builder.len() > 0 {
                    parsed_path.push_back(JsonSelectComponent::PropertyName(token_builder.clone()));
                    token_builder.clear();
                }
            }
            '[' => {
                parsed_path.push_back(JsonSelectComponent::PropertyName(token_builder.clone()));
                token_builder.clear();

                parsing_array_index = true;
            }
            ']' => {
                if !parsing_array_index {
                    panic!("unexpected array index close");
                }

                if let Ok(array_index) = token_builder.as_str().parse::<isize>() {
                    parsed_path.push_back(JsonSelectComponent::ArrayIndex(array_index));
                    token_builder.clear();
                } else {
                    panic!("array index couldn't be parsed to a number");
                }
            }
            _ => token_builder.push(c),
        }
    }
    
    if token_builder.len() > 0 {
        parsed_path.push_back(JsonSelectComponent::PropertyName(token_builder.clone()));
    }

    parsed_path
}

enum JsonPathOperator {
    ObjectRoot,
    ArrayRoot(isize),
}

enum JsonPathPartialOperator {
    None,
    Root,
    ArrayRoot,
    ArrayRootIndex(String),
    MemberAccess,
    DeepScanMemberAccess,
}

struct JsonPath {
    path: String,
    operations: Vec<JsonPathOperator>,
    partial_operation: JsonPathPartialOperator,
}

impl JsonPath {
    fn from(path: &str) -> JsonPath {
        let mut json_path = JsonPath { path: String::from(path), operations: Vec::new(), partial_operation: JsonPathPartialOperator::None };
        json_path.tokenise();

        json_path
    }

    fn tokenise(&mut self) {
        for c in self.path.clone().chars() {
            match c {
                '$' => {
                    match self.partial_operation {
                        JsonPathPartialOperator::None => self.partial_operation = JsonPathPartialOperator::Root,
                        _ => todo!(),
                    }
                }
                '.' => {
                    match self.partial_operation {
                        JsonPathPartialOperator::None => self.partial_operation = JsonPathPartialOperator::MemberAccess,
                        JsonPathPartialOperator::Root => {
                            self.operations.push(JsonPathOperator::ObjectRoot);
                            self.partial_operation = JsonPathPartialOperator::None;
                        }
                        JsonPathPartialOperator::MemberAccess => self.partial_operation = JsonPathPartialOperator::DeepScanMemberAccess,
                        _ => todo!(),
                    }
                }
                '[' => {
                    match self.partial_operation {
                        JsonPathPartialOperator::Root => self.partial_operation = JsonPathPartialOperator::ArrayRoot,
                        _ => todo!(),
                    }
                }
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::ArrayRoot => self.partial_operation = JsonPathPartialOperator::ArrayRootIndex(String::from(c)),
                        JsonPathPartialOperator::ArrayRootIndex(index) => index.push(c),
                        _ => todo!(),
                    }
                }
                ']' => {
                    match &self.partial_operation {
                        JsonPathPartialOperator::ArrayRootIndex(index) => {
                            if let Ok(i) = index.as_str().parse::<isize>() {
                                self.operations.push(JsonPathOperator::ArrayRoot(i));
                            }
                            self.partial_operation = JsonPathPartialOperator::None;
                        }
                        _ => todo!(),
                    }
                }
                _ => {}
            }
        }
    }
}

impl fmt::Display for JsonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for operation in &self.operations {
            match operation {
                JsonPathOperator::ObjectRoot => {
                    if let Err(e) = write!(f, "ObjectRoot") {
                        return Err(e);
                    }
                }
                JsonPathOperator::ArrayRoot(index) => {
                    if let Err(e) = write!(f, "ArrayRoot[{}]", index) {
                        return Err(e);
                    }
                }
            };
        }

        Ok(())
    }
}