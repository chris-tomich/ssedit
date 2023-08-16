mod json;

use clap::Parser;
use core::fmt;
use json::lexer::{JsonStreamLexer, JsonStreamStatus, JsonStreamToken, JsonTokenType};
use std::{
    collections::LinkedList,
    io::{self, BufRead, Read},
};
use strum_macros::Display;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct SSEditArgs {
    #[arg(short, long)]
    select: String,

    #[arg(short, long, default_value = "")]
    replace: String,
}

fn main() -> io::Result<()> {
    let read = false;
    let finder = true;

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
        return;
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

                json_lexer.push_char(c);

                loop {
                    match json_getter.parse(json_lexer.pop_token()) {
                        JsonStreamStatus::None => break,
                        JsonStreamStatus::Token(token) => {
                            capture_mode = true;
                            captured_tokens.push_back(token);
                        }
                        JsonStreamStatus::Finish => {
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
                                            if token.token_type == JsonTokenType::NewLine
                                                || token.token_type == JsonTokenType::Whitespace
                                            {
                                                print!("{}", token.token_raw);
                                            } else if token.token_type == JsonTokenType::StringValue
                                            {
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
                            break;
                        }
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
                let line = result.unwrap_or_else(|_| {
                    eof = true;
                    String::new()
                });

                for c in line.chars() {
                    json_lexer.push_char(c);

                    loop {
                        match json_lexer.pop_token() {
                            JsonStreamStatus::None => break,
                            JsonStreamStatus::Token(token) => tokens.push(token),
                            JsonStreamStatus::Finish => break,
                        }
                    }
                }
            }
            None => break,
        }

        json_lexer.push_char('\n');

        loop {
            match json_lexer.pop_token() {
                JsonStreamStatus::None => break,
                JsonStreamStatus::Token(token) => tokens.push(token),
                JsonStreamStatus::Finish => break,
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

fn search() {
    println!("Path '$.' parsed as '{}'", JsonPath::from("$."));
    println!("Path '$[10]' parsed as '{}'", JsonPath::from("$[10]"));
    println!(
        "Path '$[10].batters[531]' parsed as '{}'",
        JsonPath::from("$[10].batters[531]")
    );
    println!(
        "Path '$[10][531]['batters']' parsed as '{}'",
        JsonPath::from("$[10][531]['batters']")
    );
    println!(
        "Path '$.batters' parsed as '{}'",
        JsonPath::from("$.batters")
    );
    println!(
        "Path '$..batters' parsed as '{}'",
        JsonPath::from("$..batters")
    );
    println!(
        "Path '$.batters.batter' parsed as '{}'",
        JsonPath::from("$.batters.batter")
    );
    println!(
        "Path '$.batters[252]' parsed as '{}'",
        JsonPath::from("$.batters[252]")
    );
    println!(
        "Path '$.batters['batter']' parsed as '{}'",
        JsonPath::from("$.batters['batter']")
    );
    println!(
        "Path '$.batters['batter'][252]' parsed as '{}'",
        JsonPath::from("$.batters['batter'][252]")
    );
    println!(
        "Path '$['batters']['batter'][252]' parsed as '{}'",
        JsonPath::from("$['batters']['batter'][252]")
    );
    println!(
        "Path '$['batters'].batter[252]' parsed as '{}'",
        JsonPath::from("$['batters'].batter[252]")
    );
    println!(
        "Path '$['\\'batters\\''].batter[252]' parsed as '{}'",
        JsonPath::from("$['\\'batters\\''].batter[252]")
    );
    println!(
        "Path '$[\"'batters'\"].batter[252]' parsed as '{}'",
        JsonPath::from("$[\"'batters'\"].batter[252]")
    );
    println!(
        "Path '$[\"\\\"batters\\\"\"].batter[252]' parsed as '{}'",
        JsonPath::from("$[\"\\\"batters\\\"\"].batter[252]")
    );
    println!(
        "Path '$[\"'batters'\"].batter[252]' parsed as '{}'",
        JsonPath::from("$[\"'batters'\"].batter[252]")
    );
    println!(
        "Path '$[\"'batters'\"].batter[252][1:10]' parsed as '{}'",
        JsonPath::from("$[\"'batters'\"].batter[252][1:10]")
    );
    println!(
        "Path '$[\"'batters'\"].batter[252][1:10][?(@.color == 'blue')]' parsed as '{}'",
        JsonPath::from("$[\"'batters'\"].batter[252][1:10][?(@.color == 'blue')]")
    );
    println!("Path '$[\"'batters'\"].batter[252][1:10][?(@.color == 'green' || (@.color[0] == 'blue' && @.color[1] == 'yellow'))]' parsed as '{}'", JsonPath::from("$[\"'batters'\"].batter[252][1:10][?(@.color == 'green' || (@.color[0] == 'blue' && @.color[1] == 'yellow'))]"));
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
            JsonSelect {
                path: path,
                current_token: starting_token,
                parse_mode: JsonSelectParseMode::Search,
                current_search_depth: -1,
                current_capture_depth: 0,
                current_index: -1,
            }
        } else {
            panic!("path isn't supported")
        }
    }

    fn parse(&mut self, stream: JsonStreamStatus) -> JsonStreamStatus {
        match stream {
            JsonStreamStatus::None => {}
            JsonStreamStatus::Token(token) => {
                if self.process_token(&token) {
                    return JsonStreamStatus::Token(token);
                } else if self.parse_mode == JsonSelectParseMode::Finish {
                    return JsonStreamStatus::Finish;
                }
            }
            JsonStreamStatus::Finish => return stream,
        }

        JsonStreamStatus::None
    }

    fn process_token(&mut self, token: &JsonStreamToken) -> bool {
        match self.parse_mode {
            JsonSelectParseMode::Search => {
                match &self.current_token {
                    JsonSelectComponent::PropertyName(name) => match token.token_type {
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
                    },
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
                            return false;
                        }
                    }
                    JsonTokenType::ArrayOpen => self.current_capture_depth += 1,
                    JsonTokenType::ArrayClose => {
                        self.current_capture_depth -= 1;

                        if self.current_capture_depth == 0 {
                            self.parse_mode = JsonSelectParseMode::Finish;
                        } else if self.current_capture_depth == -1 {
                            self.parse_mode = JsonSelectParseMode::Finish;
                            return false;
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

enum JsonPathPartialOperator {
    Root,
    OpenRootBracket,
    ArrayRootIndex(String),
    PreMemberAccess,
    MemberAccess(String),
    DeepScanMemberAccess(String),
    OpenBracket,
    ArrayIndex(String),
    ArraySlice(String),
    OpenFilter,
    FilterExpression { depth: isize, expr: String },
    EscapeCharacter(),
    OpenSingleQuotes(String),
    OpenDoubleQuotes(String),
    ClosedSingleQuotes(String),
    ClosedDoubleQuotes(String),
}

struct JsonPath {
    path: String,
    operations: Vec<JsonPathOperator>,
    partial_operations: Vec<JsonPathPartialOperator>,
}

impl JsonPath {
    fn from(path: &str) -> JsonPath {
        let mut json_path = JsonPath {
            path: String::from(path),
            operations: Vec::new(),
            partial_operations: Vec::new(),
        };
        json_path.tokenise();

        json_path
    }

    fn tokenise(&mut self) {
        let mut terminated_path = self.path.clone();
        terminated_path.push_str("\n");

        for c in terminated_path.chars() {
            match c {
                '$' => {
                    if let None = self.partial_operations.pop() {
                        self.partial_operations.push(JsonPathPartialOperator::Root);
                    } else {
                        todo!();
                    }
                }
                '.' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => {
                                self.operations.push(JsonPathOperator::ObjectRoot);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::PreMemberAccess => self
                                .partial_operations
                                .push(JsonPathPartialOperator::DeepScanMemberAccess(String::new())),
                            JsonPathPartialOperator::MemberAccess(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                                self.partial_operations
                                    .push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(name) => {
                                self.operations
                                    .push(JsonPathOperator::DeepScanMemberAccess(name));
                                self.partial_operations
                                    .push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            _ => todo!(),
                        }
                    } else {
                        self.partial_operations
                            .push(JsonPathPartialOperator::PreMemberAccess);
                    }
                }
                '[' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => self
                                .partial_operations
                                .push(JsonPathPartialOperator::OpenRootBracket),
                            JsonPathPartialOperator::MemberAccess(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenBracket);
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            _ => self
                                .partial_operations
                                .push(JsonPathPartialOperator::OpenBracket),
                        }
                    } else {
                        self.partial_operations
                            .push(JsonPathPartialOperator::OpenBracket)
                    }
                }
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::OpenRootBracket => self
                                .partial_operations
                                .push(JsonPathPartialOperator::ArrayRootIndex(String::from(c))),
                            JsonPathPartialOperator::OpenBracket => self
                                .partial_operations
                                .push(JsonPathPartialOperator::ArrayIndex(String::from(c))),
                            JsonPathPartialOperator::ArrayRootIndex(mut index) => {
                                index.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::ArrayRootIndex(index));
                            }
                            JsonPathPartialOperator::ArrayIndex(mut index) => {
                                index.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::ArrayIndex(index));
                            }
                            JsonPathPartialOperator::ArraySlice(mut index) => {
                                index.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::ArraySlice(index));
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            _ => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                ']' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::ArrayRootIndex(index) => {
                                if let Ok(i) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayRoot(i));
                                }
                            }
                            JsonPathPartialOperator::ArrayIndex(index) => {
                                if let Ok(i) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayIndex(i));
                                }
                            }
                            JsonPathPartialOperator::ArraySlice(index) => {
                                let mut indexes = index.split(':');
                                let first = indexes.next();
                                let second = indexes.next();

                                if let Some(start) = first {
                                    if let Some(end) = second {
                                        if let Ok(start) = start.parse::<isize>() {
                                            if let Ok(end) = end.parse::<isize>() {
                                                self.operations
                                                    .push(JsonPathOperator::ArraySlice(start, end));
                                            } else {
                                                panic!("open ended slices not supported yet");
                                            }
                                        } else {
                                            panic!("open ended slices not supported yet");
                                        }
                                    } else {
                                        panic!("open ended slices not supported yet");
                                    }
                                } else {
                                    panic!("open ended slices not supported yet");
                                }
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                            }
                            JsonPathPartialOperator::ClosedDoubleQuotes(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                            }
                            _ => {}
                        }
                    }
                }
                '\'' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::OpenRootBracket => {
                                self.operations.push(JsonPathOperator::ObjectRoot);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::OpenBracket => {
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {
                                if let Some(partial_operation) = self.partial_operations.pop() {
                                    match partial_operation {
                                        JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(
                                                JsonPathPartialOperator::OpenSingleQuotes(name),
                                            );
                                        }
                                        JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(
                                                JsonPathPartialOperator::OpenDoubleQuotes(name),
                                            );
                                        }
                                        _ => panic!("unexpected character '{}'", c),
                                    }
                                }
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(name) => self
                                .partial_operations
                                .push(JsonPathPartialOperator::ClosedSingleQuotes(name)),
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            _ => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                '"' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::OpenRootBracket => {
                                self.operations.push(JsonPathOperator::ObjectRoot);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::OpenBracket => self
                                .partial_operations
                                .push(JsonPathPartialOperator::OpenDoubleQuotes(String::new())),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {
                                if let Some(partial_operation) = self.partial_operations.pop() {
                                    match partial_operation {
                                        JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(
                                                JsonPathPartialOperator::OpenSingleQuotes(name),
                                            );
                                        }
                                        JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(
                                                JsonPathPartialOperator::OpenDoubleQuotes(name),
                                            );
                                        }
                                        _ => panic!("unexpected character '{}'", c),
                                    }
                                }
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(name))
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(name) => self
                                .partial_operations
                                .push(JsonPathPartialOperator::ClosedDoubleQuotes(name)),
                            _ => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                '\\' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {}
                            JsonPathPartialOperator::OpenSingleQuotes(name) => {
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(name));
                                self.partial_operations
                                    .push(JsonPathPartialOperator::EscapeCharacter());
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(name) => {
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                                self.partial_operations
                                    .push(JsonPathPartialOperator::EscapeCharacter())
                            }
                            _ => todo!(),
                        }
                    } else {
                        todo!();
                    }
                }
                ':' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::PreMemberAccess => self
                                .partial_operations
                                .push(JsonPathPartialOperator::MemberAccess(String::from(c))),
                            JsonPathPartialOperator::MemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::MemberAccess(name));
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::DeepScanMemberAccess(name));
                            }
                            JsonPathPartialOperator::Root => todo!("{}", c),
                            JsonPathPartialOperator::OpenRootBracket => todo!("{}", c),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenBracket => self
                                .partial_operations
                                .push(JsonPathPartialOperator::ArraySlice(String::from(c))),
                            JsonPathPartialOperator::ArrayIndex(mut index) => {
                                index.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::ArraySlice(index));
                            }
                            JsonPathPartialOperator::ArraySlice(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenFilter => todo!("{}", c),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!("{}", c),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!("{}", c),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!("{}", c),
                        }
                    } else {
                        todo!("{}", c);
                    }
                }
                '?' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => todo!(),
                            JsonPathPartialOperator::OpenRootBracket => todo!(),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!(),
                            JsonPathPartialOperator::PreMemberAccess => todo!(),
                            JsonPathPartialOperator::MemberAccess(_) => todo!(),
                            JsonPathPartialOperator::DeepScanMemberAccess(_) => todo!(),
                            JsonPathPartialOperator::OpenBracket => {
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenFilter);
                            }
                            JsonPathPartialOperator::ArrayIndex(_) => todo!(),
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => todo!("{}", c),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    } else {
                        todo!("{}", c);
                    }
                }
                '(' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => todo!(),
                            JsonPathPartialOperator::OpenRootBracket => todo!(),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!(),
                            JsonPathPartialOperator::PreMemberAccess => todo!(),
                            JsonPathPartialOperator::MemberAccess(_) => todo!(),
                            JsonPathPartialOperator::DeepScanMemberAccess(_) => todo!(),
                            JsonPathPartialOperator::OpenBracket => todo!(),
                            JsonPathPartialOperator::ArrayIndex(_) => todo!(),
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => {
                                self.partial_operations.push(
                                    JsonPathPartialOperator::FilterExpression {
                                        depth: 0,
                                        expr: String::new(),
                                    },
                                );
                            }
                            JsonPathPartialOperator::FilterExpression {
                                mut depth,
                                mut expr,
                            } => {
                                expr.push(c);
                                depth += 1;
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    } else {
                        todo!("{}", c);
                    }
                }
                ')' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => todo!(),
                            JsonPathPartialOperator::OpenRootBracket => todo!(),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!(),
                            JsonPathPartialOperator::PreMemberAccess => todo!(),
                            JsonPathPartialOperator::MemberAccess(_) => todo!(),
                            JsonPathPartialOperator::DeepScanMemberAccess(_) => todo!(),
                            JsonPathPartialOperator::OpenBracket => todo!(),
                            JsonPathPartialOperator::ArrayIndex(_) => todo!(),
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => todo!(),
                            JsonPathPartialOperator::FilterExpression {
                                mut depth,
                                mut expr,
                            } => {
                                if depth > 0 {
                                    expr.push(c);
                                    depth -= 1;
                                    self.partial_operations.push(
                                        JsonPathPartialOperator::FilterExpression { depth, expr },
                                    );
                                } else {
                                    self.operations
                                        .push(JsonPathOperator::FilterExpression(expr));
                                }
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    }
                }
                '\n' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => {
                                self.operations.push(JsonPathOperator::ObjectRoot)
                            }
                            JsonPathPartialOperator::OpenRootBracket => {
                                self.operations.push(JsonPathOperator::ArrayRoot(-1))
                            }
                            JsonPathPartialOperator::ArrayRootIndex(index) => {
                                if let Ok(index) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayRoot(index));
                                }
                            }
                            JsonPathPartialOperator::PreMemberAccess => self
                                .operations
                                .push(JsonPathOperator::MemberAccess(String::new())),
                            JsonPathPartialOperator::MemberAccess(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name))
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(name) => self
                                .operations
                                .push(JsonPathOperator::DeepScanMemberAccess(name)),
                            JsonPathPartialOperator::OpenBracket => self
                                .operations
                                .push(JsonPathOperator::MemberAccess(String::new())),
                            JsonPathPartialOperator::ArrayIndex(index) => {
                                if let Ok(index) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayIndex(index));
                                }
                            }
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => todo!(),
                            JsonPathPartialOperator::FilterExpression { depth: _, expr: _ } => {
                                todo!()
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::OpenDoubleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    }
                }
                _ => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::PreMemberAccess => self
                                .partial_operations
                                .push(JsonPathPartialOperator::MemberAccess(String::from(c))),
                            JsonPathPartialOperator::MemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::MemberAccess(name));
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::DeepScanMemberAccess(name));
                            }
                            JsonPathPartialOperator::Root => todo!("{}", c),
                            JsonPathPartialOperator::OpenRootBracket => todo!("{}", c),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenBracket => todo!("{}", c),
                            JsonPathPartialOperator::ArrayIndex(_) => todo!("{}", c),
                            JsonPathPartialOperator::ArraySlice(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenFilter => todo!("{}", c),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!("{}", c),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations
                                    .push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!("{}", c),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!("{}", c),
                        }
                    } else {
                        todo!("[{}]", c);
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
