use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug)]
pub struct LexerError {
    message: String,
    position: usize,
    token_length: usize,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error at char {}: {}", self.position, self.message)
    }
}

#[allow(dead_code)]
impl LexerError {
    fn syntax_error(position: usize, token_length: usize, message: &str) -> Self {
        Self {
            message: format!("syntax_error: {}", message),
            position,
            token_length,
        }
    }

    fn semantic_error(position: usize, token_length: usize, message: &str) -> Self {
        Self {
            message: format!("semantic error: {}", message),
            position,
            token_length,
        }
    }

    fn integer_out_of_range(position: usize, actual: &str) -> Self {
        LexerError::semantic_error(
            position,
            actual.len(),
            format!(
                "integer constant should be in range [-32768, 32767], actual: {}",
                actual
            )
            .as_str(),
        )
    }

    pub fn pos(&self) -> usize {
        return self.position;
    }

    pub fn tok_length(&self) -> usize {
        return self.token_length;
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

const MAX_IDENTIFIER_LENGTH: usize = 8;
const SIMPLE_TYPES: [&'static str; 6] = ["byte", "word", "integer", "real", "char", "double"];
const KEYWORDS: [&'static str; 9] = [
    "var", "byte", "word", "integer", "real", "char", "double", "array", "of",
];

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

pub fn analyze(tokens: Vec<Token>) -> Result<HashMap<String, String>, LexerError> {
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

    let mut identifiers: HashMap<String, String> = HashMap::new();
    let mut pending_identifiers: HashSet<String> = HashSet::new();
    let mut state = State::Start;
    let mut i: usize = 0;

    let mut collecting_array_type = false;
    let mut array_type = String::new();

    let mut range_left_bound = 0;

    while (state != State::Error) && (state != State::Finish) {
        match tokens.get(i) {
            None => {
                if tokens.is_empty() {
                    return Err(LexerError::syntax_error(
                        0,
                        0, // TODO:
                        "var declaration should starts with VAR keyword",
                    ));
                }

                if let Some(last) = tokens.get(tokens.len() - 1) {
                    return Err(LexerError::syntax_error(
                        last.position + last.word.len(),
                        1,
                        "expected semicolon at the end",
                    ));
                }
            }
            Some(tok) => {
                let word = &tok.word.clone();
                let word_lower = &word.to_lowercase();

                if collecting_array_type {
                    array_type.push_str(word);
                    if word_lower == "of" {
                        array_type.push(' ');
                    }
                }

                match state {
                    State::Start => {
                        if word_lower == "var" {
                            state = State::Definition;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected `var`, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::Definition => {
                        if is_identifier(word) {
                            if word.len() > MAX_IDENTIFIER_LENGTH {
                                return Err(LexerError::semantic_error(
                                    tok.position,
                                    tok.word.len(),
                                    "identifier can't be longer than 8 characters",
                                ));
                            }
                            if is_keyword(word_lower) {
                                return Err(LexerError::semantic_error(
                                    tok.position,
                                    tok.word.len(),
                                    "identifier can't reserved word: var, real, double, etc.",
                                ));
                            }

                            if !pending_identifiers.insert(String::from(word))
                                || identifiers.contains_key(word_lower)
                            {
                                return Err(LexerError::semantic_error(
                                    tok.position,
                                    tok.word.len(),
                                    format!("identifier `{}` already taken", word).as_str(),
                                ));
                            }

                            state = State::Identifier;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("`{}` should be a valid identifier", word).as_str(),
                            ));
                        }
                    }
                    State::Identifier => {
                        if word == "," {
                            state = State::Definition
                        } else if word == ":" {
                            state = State::Type;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("`{}` should be either comma or colon", word).as_str(),
                            ));
                        }
                    }
                    State::Type => {
                        if is_simple_type(word_lower) {
                            for identifier in pending_identifiers.iter() {
                                identifiers.insert(identifier.clone(), word_lower.clone());
                            }

                            pending_identifiers = HashSet::new();

                            state = State::SimpleType;
                        } else if word_lower == "array" {
                            collecting_array_type = true;
                            array_type.push_str(word);

                            state = State::Array;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!(
                                "`{}` should be a valid type keyword: byte, word, integer, etc.",
                                word
                            )
                                .as_str(),
                            ));
                        }
                    }
                    State::SimpleType => {
                        if word == "," {
                            state = State::Definition;
                        } else if word == ";" {
                            state = State::Finish;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("`{}` should be either comma or semicolon", word).as_str(),
                            ));
                        }
                    }
                    State::Array => {
                        if word == "[" {
                            state = State::RangesStart;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected `[`, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::RangesStart => {
                        if is_integer(word) {
                            if !is_integer_in_range(word) {
                                return Err(LexerError::integer_out_of_range(tok.position, word));
                            }

                            if let Ok(value) = word.parse::<i16>() {
                                range_left_bound = value;
                            } else {
                                unreachable!()
                            }

                            state = State::FirstRangeBeginValue;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!(
                                    "expected integer constant (start of a range), found `{}`",
                                    word
                                )
                                .as_str(),
                            ));
                        }
                    }
                    State::FirstRangeBeginValue => {
                        if word == ":" {
                            state = State::FirstRangeDelimiter;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected `:`, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::FirstRangeDelimiter => {
                        if is_integer(word) {
                            if !is_integer_in_range(word) {
                                return Err(LexerError::integer_out_of_range(tok.position, word));
                            }

                            if let Ok(value) = word.parse::<i16>() {
                                if value <= range_left_bound {
                                    return Err(LexerError::semantic_error(
                                        tok.position,
                                        tok.word.len(),
                                        format!("first bound of range should be less than second")
                                            .as_str(),
                                    ));
                                }
                            } else {
                                unreachable!()
                            }

                            state = State::FirstRangeEndValue;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!(
                                    "unexpected token: expected integer constant, found `{}`",
                                    word
                                )
                                .as_str(),
                            ));
                        }
                    }
                    State::FirstRangeEndValue => {
                        if word == "," {
                            state = State::RangesDelimiter;
                        } else if word == "]" {
                            state = State::RangesEnd;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("unexpected token: expected `,`, found `{}`", word)
                                    .as_str(),
                            ));
                        }
                    }
                    State::RangesDelimiter => {
                        if is_integer(word) {
                            if !is_integer_in_range(word) {
                                return Err(LexerError::integer_out_of_range(tok.position, word));
                            }

                            if let Ok(value) = word.parse::<i16>() {
                                range_left_bound = value;
                            } else {
                                unreachable!()
                            }

                            state = State::SecondRangeBeginValue;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected integer constant, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::SecondRangeBeginValue => {
                        if word == ":" {
                            state = State::SecondRangeDelimiter;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected `:`, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::SecondRangeDelimiter => {
                        if is_integer(word) {
                            if !is_integer_in_range(word) {
                                return Err(LexerError::integer_out_of_range(tok.position, word));
                            }

                            if let Ok(value) = word.parse::<i16>() {
                                if value <= range_left_bound {
                                    return Err(LexerError::semantic_error(
                                        tok.position,
                                        tok.word.len(),
                                        format!("first bound of range should be less than second")
                                            .as_str(),
                                    ));
                                }
                            } else {
                                unreachable!()
                            }

                            state = State::SecondRangeEndValue;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected integer constant, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::SecondRangeEndValue => {
                        if word == "]" {
                            state = State::RangesEnd;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected `]`, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::RangesEnd => {
                        if word_lower == "of" {
                            state = State::Of;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected `of`, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::Of => {
                        if is_simple_type(word_lower) {
                            for identifier in pending_identifiers.iter() {
                                identifiers.insert(identifier.clone(), array_type.to_string());
                            }

                            pending_identifiers = HashSet::new();
                            array_type = String::new();
                            collecting_array_type = false;

                            state = State::ArrayType;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected on of simple types (byte, integer, real, etc), found `{}`", word).as_str(),
                            ));
                        }
                    }
                    State::ArrayType => {
                        if word == ";" {
                            state = State::Finish;
                        } else if word == "," {
                            state = State::Definition;
                        } else {
                            return Err(LexerError::syntax_error(
                                tok.position,
                                tok.word.len(),
                                format!("expected `:` or `,`, found `{}`", word).as_str(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
        i += 1;
    }

    for (identifier, typ) in &identifiers {
        println!("Identifier: {}, type: {}", identifier, typ);
    }

    Ok(identifiers)
}

fn is_integer_in_range(s: &str) -> bool {
    match s.parse::<i32>() {
        Ok(value) => value >= -32768 && value <= 32767,
        Err(_) => false,
    }
}

fn is_simple_type(s: &str) -> bool {
    for kw in SIMPLE_TYPES.iter() {
        if *kw == s {
            return true;
        }
    }
    return false;
}

fn is_keyword(s: &str) -> bool {
    for kw in KEYWORDS.iter() {
        if *kw == s {
            return true;
        }
    }
    return false;
}
