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
    query: String,
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

    Ok(())
}
