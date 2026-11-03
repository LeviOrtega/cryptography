use std::fmt;

pub const RINGS: usize = 8;
pub const ALPHABET_LEN: i32 = 26;

pub type Code = [u8; RINGS];

#[derive(Debug)]
pub struct KeyParseError(pub String);

impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid CRYPTEX_KEY: {}", self.0)
    }
}

impl std::error::Error for KeyParseError {}

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

pub fn derive_coded(password: &Code, key: &Code) -> Code {
    let mut coded: Code = [0; RINGS];
    for i in 0..RINGS {
        let v = (ALPHABET_LEN - 1) - (password[i] as i32 + key[i] as i32);
        coded[i] = v.rem_euclid(ALPHABET_LEN) as u8;
    }
    coded
}

pub fn encode(code: &Code) -> String {
    code.iter().map(|&n| (b'a' + n) as char).collect()
}

pub fn random_password() -> Code {
    let mut password: Code = [0; RINGS];
    for slot in &mut password {
        *slot = rand::random_range(0..ALPHABET_LEN as u8);
    }
    password
}
