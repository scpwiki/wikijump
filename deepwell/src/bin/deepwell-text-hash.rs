/*
 * bin/deepwell-text-hash.rs
 *
 * DEEPWELL - Wikijump API provider and database manager
 * Copyright (C) 2019-2026 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use deepwell::hash::{k12_hash, text_hash_to_hex};
use std::io::{self, BufRead, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "--batch-base64-lines") {
        return hash_batch_base64_lines();
    }

    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    let hash = k12_hash(&input);
    println!("{}", text_hash_to_hex(&hash));
    Ok(())
}

fn hash_batch_base64_lines() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let Some((id, encoded)) = line.split_once('\t') else {
            return Err("batch input line must be id<TAB>base64".into());
        };
        if id.is_empty() {
            return Err("batch input id must be non-empty".into());
        }
        let data = decode_base64(encoded)?;
        let hash = k12_hash(&data);
        println!("{}\t{}", id, text_hash_to_hex(&hash));
    }
    Ok(())
}

fn decode_base64(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const INVALID: u8 = 255;
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer = [0_u8; 4];
    let mut buffer_len = 0;
    let mut padding = 0;

    for byte in input.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => {
                padding += 1;
                0
            }
            b'\r' | b'\n' | b' ' => continue,
            _ => INVALID,
        };
        if value == INVALID || padding > 2 {
            return Err("invalid base64 batch payload".into());
        }
        buffer[buffer_len] = value;
        buffer_len += 1;
        if buffer_len == 4 {
            output.push((buffer[0] << 2) | (buffer[1] >> 4));
            if padding < 2 {
                output.push((buffer[1] << 4) | (buffer[2] >> 2));
            }
            if padding == 0 {
                output.push((buffer[2] << 6) | buffer[3]);
            }
            buffer_len = 0;
            padding = 0;
        }
    }
    if buffer_len != 0 {
        return Err("invalid base64 batch payload length".into());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::decode_base64;

    #[test]
    fn decodes_padded_base64_batch_payloads() {
        assert_eq!(decode_base64("YWxwaGE=").unwrap(), b"alpha");
        assert_eq!(decode_base64("d2lraWp1bXA=").unwrap(), b"wikijump");
    }

    #[test]
    fn rejects_invalid_base64_batch_payloads() {
        assert!(decode_base64("abc").is_err());
        assert!(decode_base64("!!!!").is_err());
    }
}
