mod json;
mod yaml;

use clap::Parser;
use std::io::{self, Read};

use json::{
    lexer::{JsonStreamLexer, JsonStreamStatus, JsonToken},
    path::JsonPath,
    query::JsonQuery,
};

use yaml::lexer::{YamlStreamLexer, YamlStreamStatus, YamlToken};

#[derive(Parser, Debug)]
#[command(help_template = "ssedit {version}\n{author-with-newline}https://github.com/chris-tomich/ssedit\n {about-section} {usage-heading} {usage} \n {all-args} {tab}")]
#[command(author, version, about)]
struct SSEditArgs {
    #[arg(short, long, help = "the elements to query using JSON path")]
    query: String,

    #[arg(
        short = 's',
        long,
        default_value_t = false,
        help = "prints out the raw symbols parsed from the input, by default the best option will be chosen"
    )]
    raw_symbols: bool,

    #[arg(
        short = 'f',
        long,
        default_value_t = String::from("json"),
        help = String::from("file type to be edited i.e. json or yaml"),
    )]
    file_type: String,
}

fn main() -> io::Result<()> {
    let args = SSEditArgs::parse();

    if args.file_type.eq_ignore_ascii_case("json") {
        json_parse(args)
    } else if args.file_type.eq_ignore_ascii_case("yaml") {
        yaml_parse(args)
    } else {
        Ok(())
    }
}

fn json_parse(args: SSEditArgs) -> io::Result<()> {
    let mut buffer = [0; 1];

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

    let mut json_lexer = JsonStreamLexer::new();

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
                                        JsonToken::NullValue(raw) => print!("{}", raw),
                                        JsonToken::UndefinedValue(raw) => print!("{}", raw),
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
                    JsonToken::NullValue(raw) => print!("{}", raw),
                    JsonToken::UndefinedValue(raw) => print!("{}", raw),
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

fn yaml_parse(args: SSEditArgs) -> io::Result<()> {
    let mut buffer = [0; 1];

    let query_path_str = if args.query.is_empty() {
        eprintln!("no select command provided");
        return Ok(());
    } else {
        args.query.as_str()
    };

    let mut yaml_lexer = YamlStreamLexer::new();

    loop {
        match io::stdin().lock().read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let c = buffer[0] as char;

                if let Err(msg) = yaml_lexer.push_char(c) {
                    panic!("{}", msg);
                }

                loop {
                    match yaml_lexer.pop_token() {
                        YamlStreamStatus::None => break,
                        YamlStreamStatus::Token(token) => {
                            if args.raw_symbols {
                                match token {
                                    YamlToken::PropertyName { raw, name } => print!("{}", raw),
                                    YamlToken::BooleanValue { raw, value } => todo!(),
                                    YamlToken::StringValue { raw, value } => todo!(),
                                    YamlToken::IntegerValue { raw, value } => todo!(),
                                    YamlToken::FloatValue { raw, value } => todo!(),
                                    YamlToken::NullValue(_) => todo!(),
                                    YamlToken::ObjectOpen(_) => todo!(),
                                    YamlToken::ObjectClose(_) => todo!(),
                                    YamlToken::ArrayOpen(_) => todo!(),
                                    YamlToken::ArrayClose(_) => todo!(),
                                    YamlToken::Whitespace(_) => todo!(),
                                    YamlToken::NewLine(raw) => print!("{}", raw),
                                    YamlToken::ArrayItemDelimiter(_) => todo!(),
                                    YamlToken::PropertyDelimiter(_) => todo!(),
                                    YamlToken::KeyValueDelimiter(_) => todo!(),
                                    YamlToken::Content(_) => todo!(),
                                    YamlToken::Paragraph(_) => todo!(),
                                    YamlToken::Line(_) => todo!(),
                                    YamlToken::ParagraphBreak(_) => todo!(),
                                    YamlToken::Alias(_) => todo!(),
                                    YamlToken::Dereference(_) => todo!(),
                                    YamlToken::Comment(_) => todo!(),
                                    YamlToken::YamlStart(raw) => print!("{}", raw),
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => todo!(),
        }
    }

    Ok(())
}
