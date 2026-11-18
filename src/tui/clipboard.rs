use std::io::{self, Write};

pub fn copy(text: &str) -> io::Result<()> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::new();
    for chunk in text.as_bytes().chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        encoded.push(ALPHABET[usize::from(first >> 2)] as char);
        encoded.push(ALPHABET[usize::from((first & 3) << 4 | second >> 4)] as char);
        encoded.push(if chunk.len() >= 2 { ALPHABET[usize::from((second & 15) << 2 | third >> 6)] as char } else { '=' });
        encoded.push(if chunk.len() == 3 { ALPHABET[usize::from(third & 63)] as char } else { '=' });
    }
    let mut output = io::stdout().lock();
    write!(output, "\x1b]52;c;{encoded}\x07")?;
    output.flush()
}
