use crate::error::{AssemblerError, AssemblerResult};
const B40_ALPHABET: &[u8; 40] = b" 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_-/";

fn encode_b40_uint(s: &[u8]) -> AssemblerResult<u32> {
    let mut result = 0u32;

    for &byte in s.iter().take(6) {
        let upper = byte.to_ascii_uppercase();
        let idx = B40_ALPHABET
            .iter()
            .position(|&c| c == upper)
            .ok_or(AssemblerError::InvalidB40Char(upper as char))?;
        result = result * 40 + idx as u32;
    }
    for _ in s.len()..6 {
        result *= 40;
    }
    Ok(result)
}
pub fn encode_b40(name: &str) -> AssemblerResult<[u8; 8]> {
    let bytes = name.as_bytes();
    let first = encode_b40_uint(&bytes[..bytes.len().min(6)])?;
    // just silently truncate longer strings TODO: explicitly define behavior
    let second = encode_b40_uint(if bytes.len() > 6 {
        &bytes[6..12.min(bytes.len())]
    } else {
        &[]
    })?;
    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&first.to_be_bytes());
    out[4..8].copy_from_slice(&second.to_be_bytes());
    Ok(out)
}
fn _decode_b40_uint(mut val: u32) -> String {
    let mut chars = [0u8; 6];
    for i in (0..6).rev() {
        let idx = (val % 40) as usize;
        chars[i] = B40_ALPHABET[idx];
        val /= 40;
    }
    unsafe { String::from_utf8_unchecked(chars.to_vec()) }
}
pub fn _decode_b40(bytes: &[u8; 8]) -> String {
    let a = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    let b = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
    (_decode_b40_uint(a) + &_decode_b40_uint(b))
        .trim_end()
        .to_string()
}
pub fn _validate_b40_chars(name: &str) -> AssemblerResult<()> {
    for byte in name.bytes() {
        let upper = byte.to_ascii_uppercase();
        if !B40_ALPHABET.contains(&upper) {
            return Err(AssemblerError::InvalidB40Char(upper as char));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b40_encode() {
        let encoded = encode_b40("EVAR01ZN01_N").unwrap();
        assert_eq!(encoded, [0x60, 0x7a, 0xed, 0x2a, 0xdf, 0x64, 0x8c, 0x60]);
    }

    #[test]
    fn test_b40_decode() {
        let decoded = _decode_b40(&[0x60, 0x7a, 0xed, 0x2a, 0xdf, 0x64, 0x8c, 0x60]);
        assert_eq!(decoded, "EVAR01ZN01_N");
    }

    #[test]
    fn test_b40_roundtrip() {
        let name = "ADD";
        let encoded = encode_b40(name).unwrap();
        let decoded = _decode_b40(&encoded);
        assert_eq!(decoded, name);
    }
    #[test]
    fn test_b40_roundtrip_full_len() {
        let name = "EVAR01ZN01_N";
        let encoded = encode_b40(name).unwrap();
        let decoded = _decode_b40(&encoded);
        assert_eq!(decoded, name);
    }

    #[test]
    fn test_b40_invalid_char() {
        let result = encode_b40("INVALID!");
        assert!(matches!(result, Err(AssemblerError::InvalidB40Char('!'))));
    }

    #[test]
    fn test_validate_b40_chars() {
        let invalid_result = _validate_b40_chars("INVALID!");
        assert!(matches!(
            invalid_result,
            Err(AssemblerError::InvalidB40Char('!'))
        ));
        let valid_result = _validate_b40_chars("VALID");
        assert!(matches!(valid_result, Ok(..)));
    }
}
