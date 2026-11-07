// Released under the MIT License.
// Copyright, 2026, by Samuel Williams.

//! The media type grammar of RFC 9110, sections 5.6 and 8.3.1.

use crate::MediaTypeError;
use std::fmt::{self, Write as _};

/// A parsed media type or range, before wildcard rules are applied. Names are lowercase.
pub(crate) struct Parsed {
    /// `type/subtype`.
    pub essence: String,
    /// The position of the `/` in `essence`.
    pub slash: usize,
    pub parameters: Vec<(String, String)>,
}

/// Whether `byte` is a `tchar`, a character permitted in a token.
pub(crate) const fn is_token_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    ) || byte.is_ascii_alphanumeric()
}

pub(crate) fn is_token(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(is_token_byte)
}

/// Whether `character` can appear in a quoted string, possibly escaped: tab, space, visible ASCII, or non-ASCII text.
pub(crate) fn is_quotable(character: char) -> bool {
    character == '\t' || !character.is_ascii_control()
}

/// Parse `type "/" subtype *( OWS ";" OWS [ parameter ] )`. Surrounding whitespace is not accepted; callers trim field values first.
pub(crate) fn parse(text: &str) -> Result<Parsed, MediaTypeError> {
    let mut scanner = Scanner { text, position: 0 };
    let top_level_type = scanner.token()?;
    scanner.expect(b'/')?;
    let subtype = scanner.token()?;
    let essence = format!("{top_level_type}/{subtype}").to_ascii_lowercase();
    let mut parameters: Vec<(String, String)> = Vec::new();
    loop {
        let before = scanner.position;
        scanner.whitespace();
        if scanner.at_end() {
            // Trailing whitespace is only permitted after a semicolon.
            if scanner.position != before {
                return Err(MediaTypeError::Syntax);
            }
            break;
        }
        scanner.expect(b';')?;
        scanner.whitespace();
        // An empty parameter, as in `text/plain;` or `text/plain;;charset=utf-8`, is permitted.
        if scanner.at_end() || matches!(scanner.peek(), Some(b';' | b' ' | b'\t')) {
            continue;
        }
        let name = scanner.token()?.to_ascii_lowercase();
        scanner.expect(b'=')?;
        let value = if scanner.peek() == Some(b'"') {
            scanner.quoted_string()?
        } else {
            scanner.token()?.to_owned()
        };
        if parameters.iter().any(|(existing, _)| *existing == name) {
            return Err(MediaTypeError::DuplicateParameter);
        }
        // Checking each new name against the earlier ones is quadratic, so bound the count before it grows.
        if parameters.len() == crate::MediaRange::PARAMETER_COUNT_LIMIT {
            return Err(MediaTypeError::TooManyParameters);
        }
        parameters.push((name, value));
    }
    Ok(Parsed {
        essence,
        slash: top_level_type.len(),
        parameters,
    })
}

/// Write a parameter value as a token, or as a quoted string when it is not one.
// SECURITY: Serialized values cannot add parameters or break a header line: values with control characters other than tab, including CR and LF, are refused when parsed or added, and every other value is written as a token or as a quoted string with `"` and `\` escaped.
pub(crate) fn write_value(formatter: &mut fmt::Formatter<'_>, value: &str) -> fmt::Result {
    if is_token(value) {
        return formatter.write_str(value);
    }
    formatter.write_char('"')?;
    for character in value.chars() {
        if matches!(character, '"' | '\\') {
            formatter.write_char('\\')?;
        }
        formatter.write_char(character)?;
    }
    formatter.write_char('"')
}

struct Scanner<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> Scanner<'a> {
    fn at_end(&self) -> bool {
        self.position == self.text.len()
    }

    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.position).copied()
    }

    fn expect(&mut self, byte: u8) -> Result<(), MediaTypeError> {
        if self.peek() != Some(byte) {
            return Err(MediaTypeError::Syntax);
        }
        self.position += 1;
        Ok(())
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.position += 1;
        }
    }

    fn token(&mut self) -> Result<&'a str, MediaTypeError> {
        let start = self.position;
        while self.peek().is_some_and(is_token_byte) {
            self.position += 1;
        }
        if self.position == start {
            return Err(MediaTypeError::Syntax);
        }
        Ok(&self.text[start..self.position])
    }

    /// `DQUOTE *( qdtext / quoted-pair ) DQUOTE`, returning the unescaped value, when the caller has seen the opening quote. Control characters other than tab, including line breaks, are rejected.
    fn quoted_string(&mut self) -> Result<String, MediaTypeError> {
        self.position += 1;
        let mut value = String::new();
        let mut characters = self.text[self.position..].char_indices();
        while let Some((offset, character)) = characters.next() {
            match character {
                '"' => {
                    self.position += offset + 1;
                    return Ok(value);
                }
                '\\' => match characters.next() {
                    Some((_, escaped)) if is_quotable(escaped) => value.push(escaped),
                    Some(_) => return Err(MediaTypeError::ParameterValue),
                    None => return Err(MediaTypeError::Syntax),
                },
                character if is_quotable(character) => value.push(character),
                _ => return Err(MediaTypeError::ParameterValue),
            }
        }
        Err(MediaTypeError::Syntax)
    }
}
