// SPDX-FileCopyrightText: 2025 Pier-Hugues Pellerin <ph@heykimo.com>
//
// SPDX-License-Identifier: MIT

#[derive(Debug, PartialEq)]
pub struct DecodingErr;

impl std::error::Error for DecodingErr {}
impl std::fmt::Display for DecodingErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Decoding error, invalid hex representation")
    }
}

pub fn decode(s: &str) -> Result<String, DecodingErr> {
    let mut new = String::new();

    for c in Decoder(s.chars()) {
        match c {
            Ok(c) => new.push(c),
            Err(err) => return Err(err),
        }
    }

    Ok(new)
}

pub struct Decoder<'a>(std::str::Chars<'a>);

impl Iterator for Decoder<'_> {
    type Item = Result<char, DecodingErr>;

    // NOTES(ph): Initially I was trying to return a Cow, but since I am decoding all the chars,
    // I need to iterated always through all of them so I am not sure this would be an economy since
    // I will still need to accumulate or not.
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some('%') => match (
                self.0.next().and_then(|v| v.to_digit(16)),
                self.0.next().and_then(|v| v.to_digit(16)),
            ) {
                (Some(a), Some(b)) => Some(char::from_u32((a << 4) | b).ok_or(DecodingErr)),
                _ => Some(Err(DecodingErr)),
            },
            Some(c) => Some(Ok(c)),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_invalid_string() {
        let candidate = "hello%-world";
        assert_eq!(decode(candidate), Err(DecodingErr));
    }

    #[test]
    fn decode_line_ending() {
        let candidate = "hello%0Aworld";
        assert_eq!(decode(candidate).unwrap(), "hello\nworld");
    }

    #[test]
    fn decode_percent() {
        let candidate = "my 100%25 percent";
        assert_eq!(decode(candidate).unwrap(), "my 100% percent");
    }

    #[test]
    fn decode_noop() {
        let candidate = "hello world";
        assert_eq!(decode(candidate).unwrap(), "hello world");
    }
}
