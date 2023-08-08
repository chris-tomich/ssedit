mod json;

use core::fmt;
use std::{io::{self, Read, BufRead}, collections::LinkedList};
use clap::Parser;
use json::lexer::{JsonStreamToken, JsonStreamLexer, JsonStream, JsonTokenType};
use strum_macros::Display;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct SSEditArgs {
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
    let args = SSEditArgs::parse();
    
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
    println!("Path '$[10].batters[531]' parsed as '{}'", JsonPath::from("$[10].batters[531]"));
    println!("Path '$[10][531]['batters']' parsed as '{}'", JsonPath::from("$[10][531]['batters']"));
    println!("Path '$.batters' parsed as '{}'", JsonPath::from("$.batters"));
    println!("Path '$..batters' parsed as '{}'", JsonPath::from("$..batters"));
    println!("Path '$.batters.batter' parsed as '{}'", JsonPath::from("$.batters.batter"));
    println!("Path '$.batters[252]' parsed as '{}'", JsonPath::from("$.batters[252]"));
    println!("Path '$.batters['batter']' parsed as '{}'", JsonPath::from("$.batters['batter']"));
    println!("Path '$.batters['batter'][252]' parsed as '{}'", JsonPath::from("$.batters['batter'][252]"));
    println!("Path '$['batters']['batter'][252]' parsed as '{}'", JsonPath::from("$['batters']['batter'][252]"));
    println!("Path '$['batters'].batter[252]' parsed as '{}'", JsonPath::from("$['batters'].batter[252]"));
    println!("Path '$['\\'batters\\''].batter[252]' parsed as '{}'", JsonPath::from("$['\\'batters\\''].batter[252]"));
    println!("Path '$[\"'batters'\"].batter[252]' parsed as '{}'", JsonPath::from("$[\"'batters'\"].batter[252]"));
    //println!("Path '$.batters.batter[2].type' parsed as '{}'", JsonPath::from("$.batters.batter[2].type"));
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
    MemberAccess(String),
    DeepScanMemberAccess(String),
    ArrayIndex(isize),
    ArraySlice(isize, isize),
    FilterExpression(String),
}

#[derive(Eq, PartialEq)]
enum JsonPathStringType {
    SingleQuotes,
    DoubleQuotes,
}

enum JsonPathPartialOperator {
    None,
    Root,
    OpenRootBracket,
    ArrayRootIndex(String),
    PreMemberAccess,
    MemberAccess(String),
    DeepScanMemberAccess(String),
    OpenBracket,
    ArrayIndex(String),
    ArraySlice(String),
    FilterExpression(String),
    EscapeCharater(JsonPathStringType,String),
    OpenSingleQuotes(String),
    OpenDoubleQuotes(String),
    ClosedSingleQuotes(String),
    ClosedDoubleQuotes(String),
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
        let mut terminated_path = self.path.clone();
        terminated_path.push_str("\n");

        for c in terminated_path.chars() {
            match c {
                '$' => {
                    match self.partial_operation {
                        JsonPathPartialOperator::None => self.partial_operation = JsonPathPartialOperator::Root,
                        _ => todo!(),
                    }
                }
                '.' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::None => self.partial_operation = JsonPathPartialOperator::PreMemberAccess,
                        JsonPathPartialOperator::Root => {
                            self.operations.push(JsonPathOperator::ObjectRoot);
                            self.partial_operation = JsonPathPartialOperator::PreMemberAccess;
                        }
                        JsonPathPartialOperator::PreMemberAccess => self.partial_operation = JsonPathPartialOperator::DeepScanMemberAccess(String::new()),
                        JsonPathPartialOperator::MemberAccess(name) => {
                            self.operations.push(JsonPathOperator::MemberAccess(std::mem::take(name)));
                            self.partial_operation = JsonPathPartialOperator::PreMemberAccess;
                        }
                        JsonPathPartialOperator::DeepScanMemberAccess(name) => {
                            self.operations.push(JsonPathOperator::DeepScanMemberAccess(std::mem::take(name)));
                            self.partial_operation = JsonPathPartialOperator::PreMemberAccess;
                        }
                        _ => todo!(),
                    }
                }
                '[' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::Root => self.partial_operation = JsonPathPartialOperator::OpenRootBracket,
                        JsonPathPartialOperator::MemberAccess(name) => {
                            self.operations.push(JsonPathOperator::MemberAccess(std::mem::take(name)));
                            self.partial_operation = JsonPathPartialOperator::OpenBracket;
                        }
                        _ => self.partial_operation = JsonPathPartialOperator::OpenBracket,
                    }
                }
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::OpenRootBracket => self.partial_operation = JsonPathPartialOperator::ArrayRootIndex(String::from(c)),
                        JsonPathPartialOperator::OpenBracket => self.partial_operation = JsonPathPartialOperator::ArrayIndex(String::from(c)),
                        JsonPathPartialOperator::ArrayRootIndex(index) => index.push(c),
                        JsonPathPartialOperator::ArrayIndex(index) => index.push(c),
                        _ => todo!(),
                    }
                }
                ']' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::ArrayRootIndex(index) => {
                            if let Ok(i) = index.as_str().parse::<isize>() {
                                self.operations.push(JsonPathOperator::ArrayRoot(i));
                            }
                            self.partial_operation = JsonPathPartialOperator::None;
                        }
                        JsonPathPartialOperator::ArrayIndex(index) => {
                            if let Ok(i) = index.as_str().parse::<isize>() {
                                self.operations.push(JsonPathOperator::ArrayIndex(i));
                            }
                            self.partial_operation = JsonPathPartialOperator::None;
                        }
                        JsonPathPartialOperator::ArraySlice(_) => todo!(),
                        JsonPathPartialOperator::FilterExpression(_) => todo!(),
                        JsonPathPartialOperator::EscapeCharater(string_type, name) => {
                            name.push(c);
                            if *string_type == JsonPathStringType::SingleQuotes {
                                self.partial_operation = JsonPathPartialOperator::OpenSingleQuotes(std::mem::take(name));
                            } else if *string_type == JsonPathStringType::DoubleQuotes {
                                self.partial_operation = JsonPathPartialOperator::OpenDoubleQuotes(std::mem::take(name));
                            } else {
                                panic!("unexpected character '{}'", c);
                            }
                        }
                        JsonPathPartialOperator::OpenSingleQuotes(name) => name.push(c),
                        JsonPathPartialOperator::OpenDoubleQuotes(name) => name.push(c),
                        JsonPathPartialOperator::ClosedSingleQuotes(name) => {
                            self.operations.push(JsonPathOperator::MemberAccess(std::mem::take(name)));
                            self.partial_operation = JsonPathPartialOperator::None;
                        }
                        JsonPathPartialOperator::ClosedDoubleQuotes(name) => {
                            self.operations.push(JsonPathOperator::MemberAccess(std::mem::take(name)));
                            self.partial_operation = JsonPathPartialOperator::None;
                        }
                        _ => {}
                    }
                }
                '\'' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::OpenRootBracket => {
                            self.operations.push(JsonPathOperator::ObjectRoot);
                            self.partial_operation = JsonPathPartialOperator::OpenSingleQuotes(String::new());
                        }
                        JsonPathPartialOperator::OpenBracket => {
                            self.partial_operation = JsonPathPartialOperator::OpenSingleQuotes(String::new());
                        }
                        JsonPathPartialOperator::FilterExpression(_) => todo!(),
                        JsonPathPartialOperator::EscapeCharater(string_type, name) => {
                            name.push(c);
                            if *string_type == JsonPathStringType::SingleQuotes {
                                self.partial_operation = JsonPathPartialOperator::OpenSingleQuotes(std::mem::take(name));
                            } else if *string_type == JsonPathStringType::DoubleQuotes {
                                self.partial_operation = JsonPathPartialOperator::OpenDoubleQuotes(std::mem::take(name));
                            } else {
                                panic!("unexpected character '{}'", c);
                            }
                        }
                        JsonPathPartialOperator::OpenSingleQuotes(name) => self.partial_operation = JsonPathPartialOperator::ClosedSingleQuotes(std::mem::take(name)),
                        JsonPathPartialOperator::OpenDoubleQuotes(name) => name.push(c),
                        _ => todo!()
                    }
                }
                '"' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::OpenRootBracket => {
                            self.operations.push(JsonPathOperator::ObjectRoot);
                            self.partial_operation = JsonPathPartialOperator::OpenDoubleQuotes(String::new());
                        }
                        JsonPathPartialOperator::OpenBracket => self.partial_operation = JsonPathPartialOperator::OpenDoubleQuotes(String::new()),
                        JsonPathPartialOperator::FilterExpression(_) => todo!(),
                        JsonPathPartialOperator::EscapeCharater(string_type, name) => {
                            name.push(c);
                            if *string_type == JsonPathStringType::SingleQuotes {
                                self.partial_operation = JsonPathPartialOperator::OpenSingleQuotes(std::mem::take(name));
                            } else if *string_type == JsonPathStringType::DoubleQuotes {
                                self.partial_operation = JsonPathPartialOperator::OpenDoubleQuotes(std::mem::take(name));
                            } else {
                                panic!("unexpected character '{}'", c);
                            }
                        }
                        JsonPathPartialOperator::OpenSingleQuotes(name) => name.push(c),
                        JsonPathPartialOperator::OpenDoubleQuotes(name) => self.partial_operation = JsonPathPartialOperator::ClosedDoubleQuotes(std::mem::take(name)),
                        _ => todo!()
                    }
                }
                '\\' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::EscapeCharater(_, _) => {}
                        JsonPathPartialOperator::OpenSingleQuotes(name) => self.partial_operation = JsonPathPartialOperator::EscapeCharater(JsonPathStringType::SingleQuotes, std::mem::take(name)),
                        JsonPathPartialOperator::OpenDoubleQuotes(name) => self.partial_operation = JsonPathPartialOperator::EscapeCharater(JsonPathStringType::DoubleQuotes, std::mem::take(name)),
                        _ => todo!()
                    }
                }
                '\n' => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::None => {}
                        JsonPathPartialOperator::Root => self.operations.push(JsonPathOperator::ObjectRoot),
                        JsonPathPartialOperator::OpenRootBracket => self.operations.push(JsonPathOperator::ArrayRoot(-1)),
                        JsonPathPartialOperator::ArrayRootIndex(index) => {
                            if let Ok(index) = index.as_str().parse::<isize>() {
                                self.operations.push(JsonPathOperator::ArrayRoot(index));
                            }
                        }
                        JsonPathPartialOperator::PreMemberAccess => self.operations.push(JsonPathOperator::MemberAccess(String::new())),
                        JsonPathPartialOperator::MemberAccess(name) => self.operations.push(JsonPathOperator::MemberAccess(std::mem::take(name))),
                        JsonPathPartialOperator::DeepScanMemberAccess(name) => self.operations.push(JsonPathOperator::DeepScanMemberAccess(std::mem::take(name))),
                        JsonPathPartialOperator::OpenBracket => self.operations.push(JsonPathOperator::MemberAccess(String::new())),
                        JsonPathPartialOperator::ArrayIndex(index) => {
                            if let Ok(index) = index.as_str().parse::<isize>() {
                                self.operations.push(JsonPathOperator::ArrayIndex(index));
                            }
                        }
                        JsonPathPartialOperator::ArraySlice(_) => todo!(),
                        JsonPathPartialOperator::FilterExpression(_) => todo!(),
                        JsonPathPartialOperator::EscapeCharater(_, _) => todo!(),
                        JsonPathPartialOperator::OpenSingleQuotes(_) => todo!(),
                        JsonPathPartialOperator::OpenDoubleQuotes(_) => todo!(),
                        JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                        JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                    }
                }
                _ => {
                    match &mut self.partial_operation {
                        JsonPathPartialOperator::PreMemberAccess => self.partial_operation = JsonPathPartialOperator::MemberAccess(String::from(c)),
                        JsonPathPartialOperator::MemberAccess(name) => name.push(c),
                        JsonPathPartialOperator::DeepScanMemberAccess(name) => name.push(c),
                        JsonPathPartialOperator::None => todo!("{}", c),
                        JsonPathPartialOperator::Root => todo!("{}", c),
                        JsonPathPartialOperator::OpenRootBracket => todo!("{}", c),
                        JsonPathPartialOperator::ArrayRootIndex(_) => todo!("{}", c),
                        JsonPathPartialOperator::OpenBracket => todo!("{}", c),
                        JsonPathPartialOperator::ArrayIndex(_) => todo!("{}", c),
                        JsonPathPartialOperator::ArraySlice(_) => todo!("{}", c),
                        JsonPathPartialOperator::FilterExpression(_) => todo!("{}", c),
                        JsonPathPartialOperator::EscapeCharater(_, _) => todo!("{}", c),
                        JsonPathPartialOperator::OpenSingleQuotes(name) => name.push(c),
                        JsonPathPartialOperator::OpenDoubleQuotes(name) => name.push(c),
                        JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!("{}", c),
                        JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!("{}", c),
                    }
                }
            }
        }
    }
}

impl fmt::Display for JsonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        let mut is_first = true;

        for operation in &self.operations {
            if !is_first {
                output.push_str(" -> ")
            } else {
                is_first = false;
            }

            match operation {
                JsonPathOperator::ObjectRoot => {
                    output.push_str("ObjectRoot");
                }
                JsonPathOperator::ArrayRoot(index) => {
                    output.push_str("ArrayRoot(");
                    output.push_str(index.to_string().as_str());
                    output.push_str(")");
                }
                JsonPathOperator::MemberAccess(name) => {
                    output.push_str("MemberAccess(");
                    output.push_str(name);
                    output.push_str(")");
                }
                JsonPathOperator::DeepScanMemberAccess(name) => {
                    output.push_str("DeepScanMemberAccess(");
                    output.push_str(name);
                    output.push_str(")");
                }
                JsonPathOperator::ArrayIndex(index) => {
                    output.push_str("ArrayIndex(");
                    output.push_str(index.to_string().as_str());
                    output.push_str(")");
                }
                JsonPathOperator::ArraySlice(start, end) => {
                    output.push_str("ArraySlice(");
                    output.push_str(start.to_string().as_str());
                    output.push_str(",");
                    output.push_str(end.to_string().as_str());
                    output.push_str(")");
                }
                JsonPathOperator::FilterExpression(filter) => {
                    output.push_str("FilterExpression(");
                    output.push_str(filter);
                    output.push_str(")");
                }
            };
        }

        write!(f, "{}", output)
    }
}