mod json;

use clap::Parser;
use core::fmt;
use std::io::{self, Read};

use json::lexer::{JsonStreamLexer, JsonStreamStatus, JsonToken};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct SSEditArgs {
    #[arg(short, long)]
    select: String,

    #[arg(short, long, default_value = "")]
    replace: String,
}

fn main() -> io::Result<()> {
    let searcher = false;
    let lexing = false;

    if searcher {
        search();
    } else if lexing {
        lexer();
    } else {
        find();
    }

    Ok(())
}

fn lexer() {
    let mut json_lexer = JsonStreamLexer::new();

    let mut buffer = [0; 1];

    let mut is_first = true;

    let mut tokenized_raw = String::new();

    loop {
        match io::stdin().lock().read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let c = buffer[0] as char;

                json_lexer.push_char(c);

                is_first = write_tokens(is_first, &mut json_lexer, &mut tokenized_raw);
            }
            Err(_) => todo!(),
        }
    }

    json_lexer.close();

    println!("\n\n{}", tokenized_raw);
}

fn write_tokens(is_first: bool, json_lexer: &mut JsonStreamLexer, tokenized_raw: &mut String) -> bool {
    let mut is_first = is_first;

    loop {
        match json_lexer.pop_token() {
            JsonStreamStatus::None => break,
            JsonStreamStatus::Token(token) => {
                if is_first {
                    is_first = false;
                } else {
                    print!(" -> ");
                }
                print!("{}", token);
                write_token(tokenized_raw, token);
            }
        }
    }

    is_first
}

fn write_token(tokenized_raw: &mut String, token: JsonToken) {
    match token {
        JsonToken::PropertyName { raw, name } => {
            tokenized_raw.push_str(raw.as_str());
            print!("({},{})", raw, name);
        }
        JsonToken::StringValue { raw, value } => {
            tokenized_raw.push_str(raw.as_str());
            print!("({},{})", raw, value);
        }
        JsonToken::IntegerValue { raw, value } => {
            tokenized_raw.push_str(raw.as_str());
            print!("({},{})", raw, value);
        }
        JsonToken::FloatValue { raw, value } => {
            tokenized_raw.push_str(raw.as_str());
            print!("({},{})", raw, value);
        }
        JsonToken::ObjectOpen(raw) => {
            tokenized_raw.push_str(raw.as_str());
            print!("({})", raw);
        }
        JsonToken::ObjectClose(raw) => {
            tokenized_raw.push_str(raw.as_str());
            print!("({})", raw);
        }
        JsonToken::ArrayOpen(raw) => {
            tokenized_raw.push_str(raw.as_str());
            print!("({})", raw);
        }
        JsonToken::ArrayClose(raw) => {
            tokenized_raw.push_str(raw.as_str());
            print!("({})", raw);
        }
        JsonToken::Whitespace(whitespace) => {
            tokenized_raw.push_str(whitespace.as_str());
            print!("({})", whitespace);
        }
        JsonToken::NewLine(newline) => {
            tokenized_raw.push_str(newline.as_str());
            print!("");
        }
        JsonToken::ArrayItemDelimiter(delimiter) => {
            tokenized_raw.push_str(delimiter.as_str());
            print!("({})", delimiter);
        }
        JsonToken::PropertyDelimiter(delimiter) => {
            tokenized_raw.push_str(delimiter.as_str());
            print!("({})", delimiter);
        }
        JsonToken::KeyValueDelimiter(delimiter) => {
            tokenized_raw.push_str(delimiter.as_str());
            print!("({})", delimiter);
        }
    }
}

fn search() {
    println!("Path '$.' tokenized as '{}'", JsonPath::from("$."));
    println!("Path '$[10]' tokenized as '{}'", JsonPath::from("$[10]"));
    println!("Path '$[10].batters[531]' tokenized as '{}'", JsonPath::from("$[10].batters[531]"));
    println!("Path '$[10][531]['batters']' tokenized as '{}'", JsonPath::from("$[10][531]['batters']"));
    println!("Path '$.batters' tokenized as '{}'", JsonPath::from("$.batters"));
    println!("Path '$..batters' tokenized as '{}'", JsonPath::from("$..batters"));
    println!("Path '$.batters.batter' tokenized as '{}'", JsonPath::from("$.batters.batter"));
    println!("Path '$.batters[252]' tokenized as '{}'", JsonPath::from("$.batters[252]"));
    println!("Path '$.batters['batter']' tokenized as '{}'", JsonPath::from("$.batters['batter']"));
    println!("Path '$.batters['batter'][252]' tokenized as '{}'", JsonPath::from("$.batters['batter'][252]"));
    println!("Path '$['batters']['batter'][252]' tokenized as '{}'", JsonPath::from("$['batters']['batter'][252]"));
    println!("Path '$['batters'].batter[252]' tokenized as '{}'", JsonPath::from("$['batters'].batter[252]"));
    println!("Path '$['\\'batters\\''].batter[252]' tokenized as '{}'", JsonPath::from("$['\\'batters\\''].batter[252]"));
    println!("Path '$[\"'batters'\"].batter[252]' tokenized as '{}'", JsonPath::from("$[\"'batters'\"].batter[252]"));
    println!("Path '$[\"\\\"batters\\\"\"].batter[252]' tokenized as '{}'", JsonPath::from("$[\"\\\"batters\\\"\"].batter[252]"));
    println!("Path '$[\"'batters'\"].batter[252]' tokenized as '{}'", JsonPath::from("$[\"'batters'\"].batter[252]"));
    println!("Path '$[\"'batters'\"].batter[252][1:10]' tokenized as '{}'", JsonPath::from("$[\"'batters'\"].batter[252][1:10]"));
    println!(
        "Path '$[\"'batters'\"].batter[252][1:10][?(@.color == 'blue')]' tokenized as '{}'",
        JsonPath::from("$[\"'batters'\"].batter[252][1:10][?(@.color == 'blue')]")
    );
    println!(
        "Path '$[\"'batters'\"].batter[252][1:10][?(@.color == 'green' || (@.color[0] == 'blue' && @.color[1] == 'yellow'))]' tokenized as '{}'",
        JsonPath::from("$[\"'batters'\"].batter[252][1:10][?(@.color == 'green' || (@.color[0] == 'blue' && @.color[1] == 'yellow'))]")
    );

    let test_path = JsonPath::from("$[\"'batters'\"].batter[252][1:10]");

    let iter = test_path.iter();

    for path in iter {
        print!("{} ", path);
    }

    println!();
}

fn find() {
    let mut json_lexer = JsonStreamLexer::new();

    let mut buffer = [0; 1];

    let test_path = JsonPath::from("$.toppings");
    let mut query = JsonQuery::from(&test_path);

    let mut capture;

    loop {
        match io::stdin().lock().read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let c = buffer[0] as char;

                json_lexer.push_char(c);

                loop {
                    match json_lexer.pop_token() {
                        JsonStreamStatus::None => break,
                        JsonStreamStatus::Token(token) => {
                            capture = query.parse(&token);

                            if capture {
                                match token {
                                    JsonToken::PropertyName { raw, name: _ } => print!("{}", raw),
                                    JsonToken::StringValue { raw, value: _ } => print!("{}", raw),
                                    JsonToken::IntegerValue { raw, value: _ } => print!("{}", raw),
                                    JsonToken::FloatValue { raw, value: _ } => print!("{}", raw),
                                    JsonToken::ObjectOpen(raw) => print!("{}", raw),
                                    JsonToken::ObjectClose(raw) => print!("{}", raw),
                                    JsonToken::ArrayOpen(raw) => print!("{}", raw),
                                    JsonToken::ArrayClose(raw) => print!("{}", raw),
                                    JsonToken::Whitespace(raw) => print!("{}", raw),
                                    JsonToken::NewLine(raw) => print!("{}", raw),
                                    JsonToken::ArrayItemDelimiter(raw) => print!("{}", raw),
                                    JsonToken::PropertyDelimiter(raw) => print!("{}", raw),
                                    JsonToken::KeyValueDelimiter(raw) => print!("{}", raw),
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => todo!(),
        }
    }

    json_lexer.close();
}

struct JsonQuery<'a> {
    path: JsonPathCursor<'a>,
    current_match_ended: bool,
    current_match_depth: isize,
}

impl<'a> JsonQuery<'a> {
    fn from(path: &'a JsonPath) -> JsonQuery {
        JsonQuery {
            path: JsonPathCursor::from(path),
            current_match_ended: false,
            current_match_depth: -1,
        }
    }

    fn parse(&mut self, token: &JsonToken) -> bool {
        let before_parse_match_state = self.path.is_matching();

        match token {
            JsonToken::PropertyName { raw: _, name } => self.path.member_access(name),
            JsonToken::ObjectOpen(_) => self.path.traverse(),
            JsonToken::ObjectClose(_) => self.path.recede(),
            JsonToken::ArrayOpen(_) => {
                self.path.traverse();
                self.path.increment_index();
            }
            JsonToken::ArrayClose(_) => self.path.recede(),
            JsonToken::ArrayItemDelimiter(_) => self.path.increment_index(),
            _ => {}
        }

        let mut is_matching = self.path.is_matching();

        let matching_just_started = !before_parse_match_state && is_matching;

        if !is_matching && self.current_match_ended {
            self.current_match_ended = false;
        }

        if is_matching {
            match token {
                JsonToken::PropertyName { raw: _, name: _ } => {
                    if self.current_match_depth <= 0 {
                        self.current_match_depth = 0;
                        is_matching = false;
                    }
                }
                JsonToken::ObjectOpen(_) => self.current_match_depth += 1,
                JsonToken::ObjectClose(_) => {
                    self.current_match_depth -= 1;

                    if self.current_match_depth == 0 {
                        self.current_match_ended = true;
                        return is_matching;
                    }
                }
                JsonToken::ArrayOpen(_) => self.current_match_depth += 1,
                JsonToken::ArrayClose(_) => {
                    self.current_match_depth -= 1;

                    if self.current_match_depth == 0 {
                        self.current_match_ended = true;
                        return is_matching;
                    }
                }
                JsonToken::ArrayItemDelimiter(_) => {
                    if self.current_match_depth <= 0 {
                        if !matching_just_started {
                            self.current_match_ended = true;
                        }
                        self.current_match_depth = 0;
                        is_matching = false;
                    }
                }
                JsonToken::PropertyDelimiter(_) => {
                    if self.current_match_depth <= 0 {
                        self.current_match_ended = true;
                        self.current_match_depth = 0;
                        is_matching = false;
                    }
                }
                JsonToken::KeyValueDelimiter(_) => {
                    if self.current_match_depth <= 0 {
                        self.current_match_depth = 0;
                        is_matching = false;
                    }
                }
                _ => {}
            }
        }

        if self.current_match_ended {
            false
        } else {
            is_matching
        }
    }
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

impl fmt::Display for JsonPathOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        match self {
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

        write!(f, "{}", output)
    }
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

struct JsonPathIterator<'a> {
    path_data: &'a Vec<JsonPathOperator>,
    current_index: usize,
}

impl<'a> JsonPathIterator<'a> {
    fn from(path: &JsonPath) -> JsonPathIterator {
        JsonPathIterator {
            path_data: &path.operations,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for JsonPathIterator<'a> {
    type Item = &'a JsonPathOperator;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.path_data.len() {
            let item = &self.path_data[self.current_index];
            self.current_index += 1;
            Some(item)
        } else {
            None
        }
    }
}

//let test_path = JsonPath::from("$.toppings.topping[1].id");

struct JsonPathCursor<'a> {
    path: &'a JsonPath,
    path_cursor: usize,
    document_cursor: usize,
    document_array_cursors: Vec<isize>,
    path_aligned: bool,
    path_match: bool,
}

impl<'a> JsonPathCursor<'a> {
    fn from(path: &'a JsonPath) -> JsonPathCursor {
        JsonPathCursor {
            path,
            path_cursor: 0,
            document_cursor: 0,
            document_array_cursors: Vec::new(),
            path_aligned: true,
            path_match: false,
        }
    }

    fn traverse(&mut self) {
        if self.path_cursor != self.document_cursor {
            self.document_cursor += 1;
            return;
        } else {
            self.document_cursor += 1;
            self.document_array_cursors.push(-1);
        }

        if !self.path_aligned {
            return;
        }

        if self.path_cursor == self.path.operations.len() - 1 {
            return;
        }

        match &self.path.operations[self.path_cursor] {
            JsonPathOperator::ObjectRoot => self.path_cursor += 1,
            JsonPathOperator::ArrayRoot(_) => todo!(),
            JsonPathOperator::MemberAccess(_) => self.path_cursor += 1,
            JsonPathOperator::DeepScanMemberAccess(_) => todo!(),
            JsonPathOperator::ArrayIndex(_) => self.path_cursor += 1,
            JsonPathOperator::ArraySlice(_, _) => todo!(),
            JsonPathOperator::FilterExpression(_) => todo!(),
        }
    }

    fn recede(&mut self) {
        if self.path_cursor != self.document_cursor {
            self.document_cursor -= 1;
            self.document_array_cursors.pop();
            return;
        } else {
            self.path_cursor -= 1;
            self.document_cursor -= 1;
            self.document_array_cursors.pop();
            self.path_aligned = true;
            self.path_match = false;
        }
    }

    fn member_access(&mut self, name: &String) {
        if self.path_cursor != self.document_cursor {
            return;
        }

        match &self.path.operations[self.path_cursor] {
            JsonPathOperator::ObjectRoot => self.path_aligned = false,
            JsonPathOperator::ArrayRoot(_) => self.path_aligned = false,
            JsonPathOperator::MemberAccess(path_member) => self.path_aligned = *name == *path_member,
            JsonPathOperator::DeepScanMemberAccess(_) => self.path_aligned = false,
            JsonPathOperator::ArrayIndex(_) => self.path_aligned = false,
            JsonPathOperator::ArraySlice(_, _) => self.path_aligned = false,
            JsonPathOperator::FilterExpression(_) => self.path_aligned = false,
        }

        self.path_match = self.path_aligned && self.path_cursor == self.path.operations.len() - 1;
    }

    fn increment_index(&mut self) {
        if self.path_cursor != self.document_cursor {
            return;
        }

        match &self.path.operations[self.path_cursor] {
            JsonPathOperator::ObjectRoot => self.path_aligned = false,
            JsonPathOperator::ArrayRoot(_) => self.path_aligned = false,
            JsonPathOperator::MemberAccess(_) => self.path_aligned = false,
            JsonPathOperator::DeepScanMemberAccess(_) => self.path_aligned = false,
            JsonPathOperator::ArrayIndex(path_index) => {
                if let Some(document_array_cursor) = self.document_array_cursors.last_mut() {
                    *document_array_cursor += 1;
                    self.path_aligned = *document_array_cursor == *path_index;
                } else {
                    self.path_aligned = false
                }
            }
            JsonPathOperator::ArraySlice(_, _) => self.path_aligned = false,
            JsonPathOperator::FilterExpression(_) => self.path_aligned = false,
        }

        self.path_match = self.path_aligned && self.path_cursor == self.path.operations.len() - 1;
    }

    fn is_matching(&self) -> bool {
        self.path_match
    }
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
        json_path.tokenize();

        json_path
    }

    fn iter(&self) -> JsonPathIterator<'_> {
        JsonPathIterator::from(self)
    }

    fn tokenize(&mut self) {
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
                                self.partial_operations.push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::PreMemberAccess => self.partial_operations.push(JsonPathPartialOperator::DeepScanMemberAccess(String::new())),
                            JsonPathPartialOperator::MemberAccess(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                                self.partial_operations.push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(name) => {
                                self.operations.push(JsonPathOperator::DeepScanMemberAccess(name));
                                self.partial_operations.push(JsonPathPartialOperator::PreMemberAccess);
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            _ => todo!(),
                        }
                    } else {
                        self.partial_operations.push(JsonPathPartialOperator::PreMemberAccess);
                    }
                }
                '[' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => self.partial_operations.push(JsonPathPartialOperator::OpenRootBracket),
                            JsonPathPartialOperator::MemberAccess(name) => {
                                self.operations.push(JsonPathOperator::MemberAccess(name));
                                self.partial_operations.push(JsonPathPartialOperator::OpenBracket);
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            _ => self.partial_operations.push(JsonPathPartialOperator::OpenBracket),
                        }
                    } else {
                        self.partial_operations.push(JsonPathPartialOperator::OpenBracket)
                    }
                }
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::OpenRootBracket => self.partial_operations.push(JsonPathPartialOperator::ArrayRootIndex(String::from(c))),
                            JsonPathPartialOperator::OpenBracket => self.partial_operations.push(JsonPathPartialOperator::ArrayIndex(String::from(c))),
                            JsonPathPartialOperator::ArrayRootIndex(mut index) => {
                                index.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::ArrayRootIndex(index));
                            }
                            JsonPathPartialOperator::ArrayIndex(mut index) => {
                                index.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::ArrayIndex(index));
                            }
                            JsonPathPartialOperator::ArraySlice(mut index) => {
                                index.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::ArraySlice(index));
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
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
                                                self.operations.push(JsonPathOperator::ArraySlice(start, end));
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
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
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
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::OpenBracket => {
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {
                                if let Some(partial_operation) = self.partial_operations.pop() {
                                    match partial_operation {
                                        JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                                        }
                                        JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                                        }
                                        _ => panic!("unexpected character '{}'", c),
                                    }
                                }
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(name) => self.partial_operations.push(JsonPathPartialOperator::ClosedSingleQuotes(name)),
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
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
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(String::new()));
                            }
                            JsonPathPartialOperator::OpenBracket => self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(String::new())),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {
                                if let Some(partial_operation) = self.partial_operations.pop() {
                                    match partial_operation {
                                        JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                                        }
                                        JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                            name.push(c);
                                            self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                                        }
                                        _ => panic!("unexpected character '{}'", c),
                                    }
                                }
                            }
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name))
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(name) => self.partial_operations.push(JsonPathPartialOperator::ClosedDoubleQuotes(name)),
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
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => {}
                            JsonPathPartialOperator::OpenSingleQuotes(name) => {
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                                self.partial_operations.push(JsonPathPartialOperator::EscapeCharacter());
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(name) => {
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                                self.partial_operations.push(JsonPathPartialOperator::EscapeCharacter())
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
                            JsonPathPartialOperator::PreMemberAccess => self.partial_operations.push(JsonPathPartialOperator::MemberAccess(String::from(c))),
                            JsonPathPartialOperator::MemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::MemberAccess(name));
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::DeepScanMemberAccess(name));
                            }
                            JsonPathPartialOperator::Root => todo!("{}", c),
                            JsonPathPartialOperator::OpenRootBracket => todo!("{}", c),
                            JsonPathPartialOperator::ArrayRootIndex(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenBracket => self.partial_operations.push(JsonPathPartialOperator::ArraySlice(String::from(c))),
                            JsonPathPartialOperator::ArrayIndex(mut index) => {
                                index.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::ArraySlice(index));
                            }
                            JsonPathPartialOperator::ArraySlice(_) => todo!("{}", c),
                            JsonPathPartialOperator::OpenFilter => todo!("{}", c),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!("{}", c),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
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
                                self.partial_operations.push(JsonPathPartialOperator::OpenFilter);
                            }
                            JsonPathPartialOperator::ArrayIndex(_) => todo!(),
                            JsonPathPartialOperator::ArraySlice(_) => todo!(),
                            JsonPathPartialOperator::OpenFilter => todo!("{}", c),
                            JsonPathPartialOperator::FilterExpression { depth, mut expr } => {
                                expr.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
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
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth: 0, expr: String::new() });
                            }
                            JsonPathPartialOperator::FilterExpression { mut depth, mut expr } => {
                                expr.push(c);
                                depth += 1;
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
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
                            JsonPathPartialOperator::FilterExpression { mut depth, mut expr } => {
                                if depth > 0 {
                                    expr.push(c);
                                    depth -= 1;
                                    self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr });
                                } else {
                                    self.operations.push(JsonPathOperator::FilterExpression(expr));
                                }
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!(),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
                            }
                            JsonPathPartialOperator::ClosedSingleQuotes(_) => todo!(),
                            JsonPathPartialOperator::ClosedDoubleQuotes(_) => todo!(),
                        }
                    }
                }
                '\n' => {
                    if let Some(partial_operation) = self.partial_operations.pop() {
                        match partial_operation {
                            JsonPathPartialOperator::Root => self.operations.push(JsonPathOperator::ObjectRoot),
                            JsonPathPartialOperator::OpenRootBracket => self.operations.push(JsonPathOperator::ArrayRoot(-1)),
                            JsonPathPartialOperator::ArrayRootIndex(index) => {
                                if let Ok(index) = index.as_str().parse::<isize>() {
                                    self.operations.push(JsonPathOperator::ArrayRoot(index));
                                }
                            }
                            JsonPathPartialOperator::PreMemberAccess => self.operations.push(JsonPathOperator::MemberAccess(String::new())),
                            JsonPathPartialOperator::MemberAccess(name) => self.operations.push(JsonPathOperator::MemberAccess(name)),
                            JsonPathPartialOperator::DeepScanMemberAccess(name) => self.operations.push(JsonPathOperator::DeepScanMemberAccess(name)),
                            JsonPathPartialOperator::OpenBracket => self.operations.push(JsonPathOperator::MemberAccess(String::new())),
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
                            JsonPathPartialOperator::PreMemberAccess => self.partial_operations.push(JsonPathPartialOperator::MemberAccess(String::from(c))),
                            JsonPathPartialOperator::MemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::MemberAccess(name));
                            }
                            JsonPathPartialOperator::DeepScanMemberAccess(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::DeepScanMemberAccess(name));
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
                                self.partial_operations.push(JsonPathPartialOperator::FilterExpression { depth, expr })
                            }
                            JsonPathPartialOperator::EscapeCharacter() => todo!("{}", c),
                            JsonPathPartialOperator::OpenSingleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenSingleQuotes(name));
                            }
                            JsonPathPartialOperator::OpenDoubleQuotes(mut name) => {
                                name.push(c);
                                self.partial_operations.push(JsonPathPartialOperator::OpenDoubleQuotes(name));
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

            output.push_str(operation.to_string().as_str());
        }

        write!(f, "{}", output)
    }
}
