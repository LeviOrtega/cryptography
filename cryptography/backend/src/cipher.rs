//! Cipher primitives for the cryptex.
//!
//! Every ring holds one of 26 letters, represented as `0..26`.
//! The puzzle: `coded[i] = (25 - (password[i] + key[i])) mod 26`,
//! so a solver recovers `password[i] = (25 - coded[i] - key[i]) mod 26`.

use std::fmt;

pub const RINGS: usize = 8;
pub const ALPHABET_LEN: i32 = 26;

/// Eight ring positions, each in `0..26`.
pub type Code = [u8; RINGS];

#[derive(Debug)]
pub struct KeyParseError(pub String);

impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid CRYPTEX_KEY: {}", self.0)
    }
}

impl std::error::Error for KeyParseError {}

/// Parse the key from its env-var form. Accepts either eight letters
/// (`HOLOGRAM`, case-insensitive) or eight comma-separated integers
/// in `0..26` (`7,14,11,14,6,17,0,12`).
pub fn parse_key(raw: &str) -> Result<Code, KeyParseError> {
    let raw = raw.trim();
    let mut key: Code = [0; RINGS];

    if raw.contains(',') {
        let parts: Vec<&str> = raw.split(',').map(str::trim).collect();
        if parts.len() != RINGS {
            return Err(KeyParseError(format!(
                "expected {RINGS} comma-separated values, got {}",
                parts.len()
            )));
        }
        for (slot, part) in key.iter_mut().zip(parts) {
            let n: i64 = part
                .parse()
                .map_err(|_| KeyParseError(format!("'{part}' is not an integer")))?;
            if !(0..ALPHABET_LEN as i64).contains(&n) {
                return Err(KeyParseError(format!("{n} is outside 0..26")));
            }
            *slot = n as u8;
        }
        return Ok(key);
    }

    let letters: Vec<char> = raw.chars().collect();
    if letters.len() != RINGS {
        return Err(KeyParseError(format!(
            "expected {RINGS} letters, got {}",
            letters.len()
        )));
    }
    for (slot, ch) in key.iter_mut().zip(letters) {
        let ch = ch.to_ascii_lowercase();
        if !ch.is_ascii_lowercase() {
            return Err(KeyParseError(format!("'{ch}' is not a letter a-z")));
        }
        *slot = (ch as u8) - b'a';
    }
    Ok(key)
}

/// Derive the displayed (coded) string from the password and the constant key.
pub fn derive_coded(password: &Code, key: &Code) -> Code {
    let mut coded: Code = [0; RINGS];
    for i in 0..RINGS {
        let v = (ALPHABET_LEN - 1) - (password[i] as i32 + key[i] as i32);
        // rem_euclid keeps the result in 0..26 even when `v` is negative,
        // which happens as soon as the key is non-zero.
        coded[i] = v.rem_euclid(ALPHABET_LEN) as u8;
    }
    coded
}

/// Render a code as lowercase letters, e.g. `[7, 4, ...]` -> `"he..."`.
pub fn encode(code: &Code) -> String {
    code.iter().map(|&n| (b'a' + n) as char).collect()
}

/// A fresh, uniformly random password.
pub fn random_password() -> Code {
    let mut password: Code = [0; RINGS];
    for slot in &mut password {
        *slot = rand::random_range(0..ALPHABET_LEN as u8);
    }
    password
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_letters_and_numbers() {
        assert_eq!(parse_key("HOLOGRAM").unwrap(), [7, 14, 11, 14, 6, 17, 0, 12]);
        assert_eq!(parse_key("7,14,11,14,6,17,0,12").unwrap(), [7, 14, 11, 14, 6, 17, 0, 12]);
        assert!(parse_key("TOOSHORT!").is_err());
        assert!(parse_key("1,2,3").is_err());
        assert!(parse_key("1,2,3,4,5,6,7,99").is_err());
    }

    #[test]
    fn coded_is_always_in_range_and_invertible() {
        let key = parse_key("ZZZZZZZZ").unwrap();
        for _ in 0..1000 {
            let p = random_password();
            let c = derive_coded(&p, &key);
            for i in 0..RINGS {
                assert!(c[i] < 26);
                let back = (25 - c[i] as i32 - key[i] as i32).rem_euclid(26) as u8;
                assert_eq!(back, p[i]);
            }
        }
    }

    #[test]
    fn encode_roundtrip() {
        assert_eq!(encode(&[0, 1, 2, 3, 4, 5, 6, 7]), "abcdefgh");
    }
}
