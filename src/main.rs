mod json;

use clap::Parser;
use std::io::{self, Read};

use json::{
    lexer::{JsonStreamLexer, JsonStreamStatus, JsonToken},
    path::JsonPath,
    query::JsonQuery,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct SSEditArgs {
    #[arg(short, long)]
    select: String,

    #[arg(short, long, default_value = "")]
    replace: String,
}

fn main() -> io::Result<()> {
    let lexing = false;

    if lexing {
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
