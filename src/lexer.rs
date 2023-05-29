use thiserror::Error;

/// The lexed token.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub start: u32,
    pub len: u16,
    pub kind: TokenKind,
}

/// The kind of `Token`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    SelectorPath,
    QuotedString,
    Number,
    BooleanTrue,
    BooleanFalse,
    Null,
    Equals,
    Add,
    Subtract,
    Multiply,
    Divide,
    Gt,
    Gte,
    Lt,
    Lte,
    Not,
    And,
    Or,
    Contains,
    ContainsAny,
    ContainsAll,
    In,
    Between,
    StartsWith,
    EndsWith,
    OpenBracket,
    CloseBracket,
    Comma,
    OpenParen,
    CloseParen,
    Coerce,
    Identifier,
}

pub struct Tokenizer<'a> {
    pos: u32,
    remaining: &'a [u8],
}

impl<'a> Tokenizer<'a> {
    /// Creates a new `Tokenizer` to iterate over tokens
    #[inline]
    #[must_use]
    pub fn new(src: &'a str) -> Self {
        Self::new_bytes(src.as_bytes())
    }

    /// Creates a new `Tokenizer` to iterate over tokens using bytes as the source.
    #[must_use]
    pub fn new_bytes(src: &'a [u8]) -> Self {
        Self {
            pos: 0,
            remaining: src,
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();

        if self.remaining.is_empty() {
            Ok(None)
        } else {
            let (kind, bytes_read) = tokenize_single_token(self.remaining)?;
            let token = Token {
                kind,
                start: self.pos,
                len: bytes_read,
            };
            self.chomp(bytes_read);
            Ok(Some(token))
        }
    }

    fn skip_whitespace(&mut self) {
        let skipped = skip_whitespace(self.remaining);
        self.chomp(skipped);
    }

    fn chomp(&mut self, len: u16) {
        self.remaining = &self.remaining[len as usize..];
        self.pos += u32::from(len);
    }
}

impl Iterator for Tokenizer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token().transpose()
    }
}

#[inline]
fn skip_whitespace(data: &[u8]) -> u16 {
    take_while(data, |c| c.is_ascii_whitespace()).unwrap_or(0)
}

#[inline]
/// Consumes bytes while a predicate evaluates to true.
fn take_while<F>(data: &[u8], mut pred: F) -> Option<u16>
    where
        F: FnMut(u8) -> bool,
{
    let mut current_index = 0;

    for b in data {
        if !pred(*b) {
            break;
        }
        current_index += 1;
    }

    if current_index == 0 {
        None
    } else {
        Some(current_index)
    }
}

/// Result of a single tokenization attempt.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the lexer.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),

    #[error("invalid number: {0}")]
    InvalidNumber(String),

    #[error("invalid boolean: {0}")]
    InvalidBool(String),

    #[error("invalid keyword: {0}")]
    InvalidKeyword(String),

    #[error("Unsupported Character `{0}`")]
    UnsupportedCharacter(u8),

    #[error("Unterminated string `{0}`")]
    UnterminatedString(String),
}

/// Try to lex a single token from the input stream.
fn tokenize_single_token(data: &[u8]) -> Result<(TokenKind, u16)> {
    let Some(b) = data.first() else {
        panic!("invalid data passed")
    };

    let (token, end) = match b {
        b'=' if data.get(1) == Some(&b'=') => (TokenKind::Equals, 2),
        b'=' => (TokenKind::Equals, 1),
        b'+' => {
            if data.get(1).map_or_else(|| false, u8::is_ascii_digit) {
                tokenize_number(data)?
            } else {
                (TokenKind::Add, 1)
            }
        }
        b'-' => {
            if data.get(1).map_or_else(|| false, u8::is_ascii_digit) {
                tokenize_number(data)?
            } else {
                (TokenKind::Subtract, 1)
            }
        }
        b'*' => (TokenKind::Multiply, 1),
        b'/' => (TokenKind::Divide, 1),
        b'>' if data.get(1) == Some(&b'=') => (TokenKind::Gte, 2),
        b'>' => (TokenKind::Gt, 1),
        b'<' if data.get(1) == Some(&b'=') => (TokenKind::Lte, 2),
        b'<' => (TokenKind::Lt, 1),
        b'(' => (TokenKind::OpenParen, 1),
        b')' => (TokenKind::CloseParen, 1),
        b'[' => (TokenKind::OpenBracket, 1),
        b']' => (TokenKind::CloseBracket, 1),
        b',' => (TokenKind::Comma, 1),
        b'!' => (TokenKind::Not, 1),
        b'"' | b'\'' => tokenize_string(data, *b)?,
        b'.' => tokenize_selector_path(data)?,
        b't' | b'f' => tokenize_bool(data)?,
        b'&' if data.get(1) == Some(&b'&') => (TokenKind::And, 2),
        b'|' if data.get(1) == Some(&b'|') => (TokenKind::Or, 2),
        b'O' => tokenize_keyword(data, "OR".as_bytes(), TokenKind::Or)?,
        b'C' => {
            if data.get(2) == Some(&b'N') {
                // can be one of CONTAINS, CONTAINS_ANY, CONTAINS_ALL
                if data.get(8) == Some(&b'_') {
                    if data.get(10) == Some(&b'N') {
                        tokenize_keyword(data, "CONTAINS_ANY".as_bytes(), TokenKind::ContainsAny)?
                    } else {
                        tokenize_keyword(data, "CONTAINS_ALL".as_bytes(), TokenKind::ContainsAll)?
                    }
                } else {
                    tokenize_keyword(data, "CONTAINS".as_bytes(), TokenKind::Contains)?
                }
            } else {
                tokenize_keyword(data, "COERCE".as_bytes(), TokenKind::Coerce)?
            }
        }
        b'I' => tokenize_keyword(data, "IN".as_bytes(), TokenKind::In)?,
        b'S' => tokenize_keyword(data, "STARTS_WITH".as_bytes(), TokenKind::StartsWith)?,
        b'E' => tokenize_keyword(data, "ENDS_WITH".as_bytes(), TokenKind::EndsWith)?,
        b'B' => tokenize_keyword(data, "BETWEEN".as_bytes(), TokenKind::Between)?,
        b'N' => tokenize_null(data)?,
        b'_' => tokenize_identifier(data)?,
        b'0'..=b'9' => tokenize_number(data)?,
        _ => return Err(Error::UnsupportedCharacter(*b)),
    };
    Ok((token, end))
}

#[inline]
fn tokenize_identifier(data: &[u8]) -> Result<(TokenKind, u16)> {
    // TODO: take until end underscore found!
    match take_while(data, |c| {
        !c.is_ascii_whitespace() && c != b')' && c != b']' && c != b','
    }) {
        // identifier must start and end with underscore
        Some(end) if end > 0 && data.get(end as usize - 1) == Some(&b'_') => {
            Ok((TokenKind::Identifier, end))
        }
        _ => Err(Error::InvalidIdentifier(
            String::from_utf8_lossy(data).to_string(),
        )),
    }
}

#[inline]
fn tokenize_string(data: &[u8], quote: u8) -> Result<(TokenKind, u16)> {
    let mut last_backslash = false;
    let mut ended_with_terminator = false;

    match take_while(&data[1..], |c| match c {
        b'\\' => {
            last_backslash = true;
            true
        }
        _ if c == quote => {
            if last_backslash {
                last_backslash = false;
                true
            } else {
                ended_with_terminator = true;
                false
            }
        }
        _ => {
            last_backslash = false;
            true
        }
    }) {
        Some(end) => {
            if ended_with_terminator {
                Ok((TokenKind::QuotedString, end + 2))
            } else {
                Err(Error::UnterminatedString(
                    String::from_utf8_lossy(data).to_string(),
                ))
            }
        }
        None => {
            if !ended_with_terminator || data.len() < 2 {
                Err(Error::UnterminatedString(
                    String::from_utf8_lossy(data).to_string(),
                ))
            } else {
                Ok((TokenKind::QuotedString, 2))
            }
        }
    }
}

#[inline]
fn tokenize_selector_path(data: &[u8]) -> Result<(TokenKind, u16)> {
    match take_while(&data[1..], |c| {
        !c.is_ascii_whitespace() && c != b')' && c != b']'
    }) {
        Some(end) => Ok((TokenKind::SelectorPath, end + 1)),
        None => Err(Error::InvalidIdentifier(
            String::from_utf8_lossy(data).to_string(),
        )),
    }
}

#[inline]
fn tokenize_bool(data: &[u8]) -> Result<(TokenKind, u16)> {
    match take_while(data, |c| c.is_ascii_alphabetic()) {
        Some(end) => match data[..end as usize] {
            [b't', b'r', b'u', b'e'] => Ok((TokenKind::BooleanTrue, end)),
            [b'f', b'a', b'l', b's', b'e'] => Ok((TokenKind::BooleanFalse, end)),
            _ => Err(Error::InvalidBool(
                String::from_utf8_lossy(data).to_string(),
            )),
        },
        None => Err(Error::InvalidBool(
            String::from_utf8_lossy(data).to_string(),
        )),
    }
}

#[inline]
fn tokenize_keyword(data: &[u8], keyword: &[u8], kind: TokenKind) -> Result<(TokenKind, u16)> {
    match take_while(data, |c| !c.is_ascii_whitespace()) {
        Some(end) if data.len() > keyword.len() && &data[..end as usize] == keyword => {
            Ok((kind, end))
        }
        _ => Err(Error::InvalidKeyword(
            String::from_utf8_lossy(data).to_string(),
        )),
    }
}

#[inline]
fn tokenize_null(data: &[u8]) -> Result<(TokenKind, u16)> {
    match take_while(data, |c| c.is_ascii_alphabetic()) {
        Some(end) if data[..end as usize] == [b'N', b'U', b'L', b'L'] => Ok((TokenKind::Null, end)),
        _ => Err(Error::InvalidKeyword(
            String::from_utf8_lossy(data).to_string(),
        )),
    }
}

#[inline]
fn tokenize_number(data: &[u8]) -> Result<(TokenKind, u16)> {
    let mut dot_seen = false;
    let mut bad_number = false;

    match take_while(data, |c| match c {
        b'.' => {
            if dot_seen {
                bad_number = true;
                false
            } else {
                dot_seen = true;
                true
            }
        }
        b'-' | b'+' | b'e' => true,
        _ => c.is_ascii_digit(),
    }) {
        Some(end) if !bad_number => Ok((TokenKind::Number, end)),
        _ => Err(Error::InvalidNumber(
            String::from_utf8_lossy(data).to_string(),
        )),
    }
}