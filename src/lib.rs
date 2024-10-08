use std::fmt;

pub struct LexerError {
    message: String,
    position: u8,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error at char {}: {}", self.position, self.message)
    }
}

#[allow(dead_code)]
impl LexerError {
    fn new(position: u8, message: &str) -> Self {
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

pub fn is_identifier(_str: &str) -> bool {
    return true;
}
