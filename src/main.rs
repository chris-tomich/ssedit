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
    #[arg(short, long, help = "the elements to query for using JSON path")]
    query: String,

    #[arg(
        short = 's',
        long,
        default_value_t = false,
        help = "prints out the raw symbols parsed from the input, by default the best option will be chosen"
    )]
    raw_symbols: bool,
}

fn main() -> io::Result<()> {
    let mut json_lexer = JsonStreamLexer::new();

    let mut buffer = [0; 1];

    let args = SSEditArgs::parse();

    let query_path_str = if args.query.is_empty() {
        eprintln!("no select command provided");
        return Ok(());
    } else {
        args.query.as_str()
    };

    let query_path = JsonPath::from(query_path_str);
    let mut query = JsonQuery::from(&query_path);

    let mut capture;

    let mut captured_tokens = Vec::new();
    let mut number_of_values = 0;

    loop {
        match io::stdin().lock().read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let c = buffer[0] as char;

                if let Err(msg) = json_lexer.push_char(c) {
                    panic!("{}", msg);
                }

                loop {
                    match json_lexer.pop_token() {
                        JsonStreamStatus::None => break,
                        JsonStreamStatus::Token(token) => {
                            capture = query.parse(&token);

                            if capture {
                                if args.raw_symbols {
                                    match token {
                                        JsonToken::PropertyName { raw, name: _ } => print!("{}", raw),
                                        JsonToken::BooleanValue { raw, value: _ } => print!("{}", raw),
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
                                } else {
                                    match token {
                                        JsonToken::StringValue { raw: _, value: _ } => number_of_values += 1,
                                        JsonToken::IntegerValue { raw: _, value: _ } => number_of_values += 1,
                                        JsonToken::FloatValue { raw: _, value: _ } => number_of_values += 1,
                                        _ => {}
                                    }

                                    captured_tokens.push(token);
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => todo!(),
        }
    }

    if !args.raw_symbols {
        if number_of_values <= 1 {
            for token in captured_tokens {
                match token {
                    JsonToken::StringValue { raw: _, value } => print!("{}", value),
                    JsonToken::IntegerValue { raw: _, value } => print!("{}", value),
                    JsonToken::FloatValue { raw: _, value } => print!("{}", value),
                    _ => {}
                }
            }
        } else {
            for token in captured_tokens {
                match token {
                    JsonToken::PropertyName { raw, name: _ } => print!("{}", raw),
                    JsonToken::BooleanValue { raw, value: _ } => print!("{}", raw),
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

    json_lexer.close();

    Ok(())
}
