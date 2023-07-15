mod json;

use std::io::{self, BufRead};
use json::lexer::{JsonStreamToken, JsonStreamLexer, JsonStream, JsonTokenType};

fn main() -> io::Result<()> {
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

    Ok(())
}