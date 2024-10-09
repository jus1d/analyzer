use std::fmt;

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

const KEYWORDS: [&'static str; 6] = ["byte", "word", "integer", "real", "char", "double"];

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
        List,
        Identifiers,
        Type,
        TypeDelimiter,
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
                            state = State::List;
                        } else {
                            return Err(LexerError::new(
                                tok.position,
                                format!("unexpected token: expected `var`, found `{}`", tok.word)
                                    .as_str(),
                            ));
                        }
                    }
                    State::List => {
                        if is_identifier(&tok.word) {
                            state = State::Identifiers;
                        } else {
                            return Err(LexerError::new(
                                tok.position,
                                format!(
                                    "unexpected token: `{}` should be a valid identifier",
                                    tok.word
                                )
                                .as_str(),
                            ));
                        }
                    }
                    State::Identifiers => {
                        if tok.word == "," {
                            state = State::List
                        } else if tok.word == ":" {
                            state = State::Type;
                        } else {
                            return Err(LexerError::new(
                                tok.position,
                                format!(
                                    "unexpected token: `{}` should be either comma or colon",
                                    tok.word
                                )
                                .as_str(),
                            ));
                        }
                    }
                    State::Type => {
                        if is_type_keyword(&tok.word) {
                            state = State::TypeDelimiter;
                        } else if tok.word == "array" {
                            todo!()
                        } else {
                            return Err(LexerError::new(
                                tok.position,
                                format!(
                                    "unexpected token: `{}` should be a valid type keyword: byte, word, integer, etc.",
                                    tok.word
                                )
                                .as_str(),
                            ));
                        }
                    }
                    State::TypeDelimiter => {
                        if tok.word == "," {
                            state = State::List;
                        } else if tok.word == ";" {
                            state = State::Finish;
                        } else {
                            return Err(LexerError::new(
                                tok.position,
                                format!(
                                    "unexpected token: `{}` should be either comma or semicolon",
                                    tok.word
                                )
                                .as_str(),
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

fn is_type_keyword(s: &str) -> bool {
    for kw in KEYWORDS.iter() {
        if *kw == s {
            return true;
        }
    }
    return false;
}
