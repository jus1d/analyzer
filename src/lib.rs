use std::{
    fmt::{self, format},
    mem,
};

#[derive(Debug)]
pub struct LexerError {
    message: String,
    position: usize,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error at char {}: {}", self.position, self.message)
    }
}

#[allow(dead_code)]
impl LexerError {
    fn new(position: usize, message: &str) -> Self {
        Self {
            message: message.to_string(),
            position,
        }
    }

    fn unexpected_token(position: usize, message: &str) -> Self {
        Self {
            message: format!("unexpected token: {}", message),
            position,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Token {
    word: String,
    position: usize,
}

impl Token {
    fn new(word: &str, position: usize) -> Self {
        Self {
            word: String::from(word),
            position,
        }
    }
}

const SIMPLE_TYPES: [&'static str; 6] = ["byte", "word", "integer", "real", "char", "double"];

pub fn tokenize(content: String) -> Result<Vec<Token>, LexerError> {
    let mut tokens: Vec<Token> = vec![];

    let mut cur = String::new();

    for (idx, ch) in content.chars().enumerate() {
        if ch == ' ' {
            if !cur.is_empty() {
                tokens.push(Token::new(cur.as_str(), idx - cur.len()));
            }
            cur = String::new();
            continue;
        } else if !ch.is_alphanumeric() {
            if !cur.is_empty() {
                tokens.push(Token::new(cur.as_str(), idx - cur.len()));
            }
            cur = String::new();
            tokens.push(Token::new(&ch.to_string(), idx));
        }

        if ch.is_alphanumeric() {
            cur.push(ch);
        }
    }

    if !cur.is_empty() {
        tokens.push(Token::new(cur.as_str(), content.len() - cur.len()));
    }

    Ok(tokens)
}

pub fn is_integer(str: &str) -> bool {
    #[derive(Debug, PartialEq)]
    enum State {
        Start,
        Digits,
        Uint,
        Error,
        Finish,
    }

    let mut state = State::Start;
    let mut i: usize = 0;

    while (state != State::Error) && (state != State::Finish) {
        match str.chars().nth(i) {
            Some(ch) => match state {
                State::Start => {
                    if ch == '-' || ch == '+' {
                        state = State::Uint;
                    } else if ch.is_ascii_digit() {
                        state = State::Digits;
                    } else {
                        state = State::Error;
                        break;
                    }
                }
                State::Uint => {
                    if ch.is_ascii_digit() {
                        state = State::Digits;
                    } else {
                        state = State::Error;
                        break;
                    }
                }
                State::Digits => {
                    if ch.is_ascii_digit() {
                        state = State::Digits;
                    } else {
                        state = State::Error;
                        break;
                    }
                }
                _ => {}
            },
            None => {
                if state == State::Digits {
                    state = State::Finish;
                } else {
                    state = State::Error;
                }
            }
        }
        i += 1;
    }

    return state == State::Finish && i == str.len() + 1;
}

pub fn is_identifier(str: &str) -> bool {
    #[derive(Debug, PartialEq)]
    enum State {
        Start,
        Chars,
        Error,
        Finish,
    }

    let mut state = State::Start;
    let mut i: usize = 0;

    while (state != State::Error) && (state != State::Finish) {
        match str.chars().nth(i) {
            Some(ch) => match state {
                State::Start => {
                    if ch.is_alphabetic() {
                        state = State::Chars;
                    } else {
                        state = State::Error;
                        break;
                    }
                }
                State::Chars => {
                    if ch.is_alphabetic() || ch.is_ascii_digit() {
                        state = State::Chars;
                    } else {
                        state = State::Error;
                        break;
                    }
                }
                _ => {}
            },
            None => {
                if state == State::Chars {
                    state = State::Finish;
                } else {
                    state = State::Error;
                }
            }
        }
        i += 1;
    }

    return state == State::Finish && i == str.len() + 1;
}

pub fn analyze(tokens: Vec<Token>) -> Result<(), LexerError> {
    #[derive(Debug, PartialEq)]
    enum State {
        Start,
        Definition,
        Identifier,
        Type,
        SimpleType,
        Array,
        RangesStart,
        RangesEnd,
        FirstRangeBeginValue,
        FirstRangeDelimiter,
        FirstRangeEndValue,
        RangesDelimiter,
        SecondRangeBeginValue,
        SecondRangeDelimiter,
        SecondRangeEndValue,
        Of,
        ArrayType,
        Error,
        Finish,
    }

    let mut state = State::Start;
    let mut i: usize = 0;

    while (state != State::Error) && (state != State::Finish) {
        match tokens.get(i) {
            Some(tok) => {
                println!("inspecting token: {}", tok.word);

                match state {
                    State::Start => {
                        if tok.word == "var" {
                            state = State::Definition;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected `var`, found `{}`", tok.word).as_str(),
                            ));
                        }
                    }
                    State::Definition => {
                        if is_identifier(&tok.word) {
                            state = State::Identifier;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("`{}` should be a valid identifier", tok.word).as_str(),
                            ));
                        }
                    }
                    State::Identifier => {
                        if tok.word == "," {
                            state = State::Definition
                        } else if tok.word == ":" {
                            state = State::Type;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("`{}` should be either comma or colon", tok.word).as_str(),
                            ));
                        }
                    }
                    State::Type => {
                        if is_simple_type(&tok.word) {
                            state = State::SimpleType;
                        } else if tok.word == "array" {
                            state = State::Array;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("`{}` should be a valid type keyword: byte, word, integer, etc.", tok.word).as_str(),
                            ));
                        }
                    }
                    State::SimpleType => {
                        if tok.word == "," {
                            state = State::Definition;
                        } else if tok.word == ";" {
                            state = State::Finish;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("`{}` should be either comma or semicolon", tok.word)
                                    .as_str(),
                            ));
                        }
                    }
                    State::Array => {
                        if tok.word == "[" {
                            state = State::RangesStart;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected `[`, found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    State::RangesStart => {
                        if tok.word == "]" {
                            state = State::RangesEnd;
                        } else if is_integer(&tok.word) {
                            state = State::FirstRangeBeginValue;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected `]` or integer constant, found '{}'", tok.word)
                                    .as_str(),
                            ));
                        }
                    }
                    State::FirstRangeBeginValue => {
                        if tok.word == ":" {
                            state = State::FirstRangeDelimiter;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected `:`, found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    State::FirstRangeDelimiter => {
                        if is_integer(&tok.word) {
                            state = State::FirstRangeEndValue;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!(
                                    "unexpected token: expected integer constant, found '{}'",
                                    tok.word
                                )
                                .as_str(),
                            ));
                        }
                    }
                    State::FirstRangeEndValue => {
                        if tok.word == "," {
                            state = State::RangesDelimiter;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("unexpected token: expected `,`, found '{}'", tok.word)
                                    .as_str(),
                            ));
                        }
                    }
                    State::RangesDelimiter => {
                        if is_integer(&tok.word) {
                            state = State::SecondRangeBeginValue;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected integer constant, found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    State::SecondRangeBeginValue => {
                        if tok.word == ":" {
                            state = State::SecondRangeDelimiter;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected `:`, found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    State::SecondRangeDelimiter => {
                        if is_integer(&tok.word) {
                            state = State::SecondRangeEndValue;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected integer constant, found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    State::SecondRangeEndValue => {
                        if tok.word == "]" {
                            state = State::RangesEnd;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected `]`, found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    State::RangesEnd => {
                        if tok.word == "of" {
                            state = State::Of;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected `of`, found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    State::Of => {
                        if is_simple_type(&tok.word) {
                            state = State::ArrayType;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected on of simple types (byte, integer, real, etc), found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    State::ArrayType => {
                        if tok.word == ";" {
                            state = State::Finish;
                        } else if tok.word == "," {
                            state = State::Definition;
                        } else {
                            return Err(LexerError::unexpected_token(
                                tok.position,
                                format!("expected `:` or `,`, found '{}'", tok.word).as_str(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            None => {
                if let Some(last) = tokens.get(tokens.len() - 1) {
                    return Err(LexerError::new(
                        last.position + last.word.len(),
                        "expected semicolon at the end",
                    ));
                }
                panic!();
            }
        }
        i += 1;
    }

    Ok(())
}

fn is_simple_type(s: &str) -> bool {
    for kw in SIMPLE_TYPES.iter() {
        if *kw == s {
            return true;
        }
    }
    return false;
}
