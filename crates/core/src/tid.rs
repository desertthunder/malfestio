//! TID (Timestamp Identifier) generation for AT Protocol.
//!
//! TIDs are used as record keys in the AT Protocol. They are 13-character
//! base32-sortable strings derived from timestamps with a clock identifier.
//!
//! Format: 13 characters encoding 64 bits:
//! - 53 bits: microseconds since Unix epoch
//! - 10 bits: clock identifier (for uniqueness within same microsecond)
//! - 1 bit: always 0 (reserved)

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Base32 "sort" alphabet used by AT Protocol TIDs.
/// This alphabet maintains lexicographic sorting.
const BASE32_SORT: &[u8; 32] = b"234567abcdefghijklmnopqrstuvwxyz";

/// Atomic counter for clock identifier within same microsecond.
static CLOCK_ID: AtomicU64 = AtomicU64::new(0);
static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// Generate a new TID.
///
/// TIDs are guaranteed to be:
/// - Unique within this process
/// - Lexicographically sortable by creation time
/// - Compatible with AT Protocol record key requirements
pub fn generate_tid() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros() as u64;

    let last = LAST_TIMESTAMP.load(Ordering::SeqCst);
    let clock_id = if now == last {
        CLOCK_ID.fetch_add(1, Ordering::SeqCst) & 0x3FF
    } else {
        LAST_TIMESTAMP.store(now, Ordering::SeqCst);
        CLOCK_ID.store(1, Ordering::SeqCst);
        0
    };

    let combined = (now << 11) | (clock_id << 1);
    encode_base32_sort(combined)
}

/// Encode a 64-bit value as a 13-character base32-sort string.
fn encode_base32_sort(mut value: u64) -> String {
    let mut result = [0u8; 13];

    for i in (0..13).rev() {
        result[i] = BASE32_SORT[(value & 0x1F) as usize];
        value >>= 5;
    }

    String::from_utf8(result.to_vec()).expect("Base32 encoding produced invalid UTF-8")
}

/// Parse a TID string and extract the timestamp.
///
/// Returns the Unix timestamp in microseconds, or None if invalid.
pub fn parse_tid_timestamp(tid: &str) -> Option<u64> {
    if tid.len() != 13 {
        return None;
    }

    let decoded = decode_base32_sort(tid)?;
    Some(decoded >> 11)
}

/// Decode a base32-sort string to a 64-bit value.
fn decode_base32_sort(s: &str) -> Option<u64> {
    let mut value: u64 = 0;

    for c in s.chars() {
        let idx = BASE32_SORT.iter().position(|&b| b == c as u8)?;
        value = (value << 5) | (idx as u64);
    }

    Some(value)
}

/// Validate that a string is a valid TID format.
pub fn is_valid_tid(tid: &str) -> bool {
    if tid.len() != 13 {
        return false;
    }

    tid.chars().all(|c| BASE32_SORT.contains(&(c as u8)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tid_length() {
        let tid = generate_tid();
        assert_eq!(tid.len(), 13);
    }

    #[test]
    fn test_tid_characters() {
        let tid = generate_tid();
        for c in tid.chars() {
            assert!(BASE32_SORT.contains(&(c as u8)), "Invalid character '{}' in TID", c);
        }
    }

    #[test]
    fn test_tid_uniqueness() {
        let tids: Vec<String> = (0..100).map(|_| generate_tid()).collect();
        let mut unique = tids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(tids.len(), unique.len(), "TIDs should be unique");
    }

    #[test]
    fn test_tid_sortability() {
        let tid1 = generate_tid();
        std::thread::sleep(std::time::Duration::from_micros(10));
        let tid2 = generate_tid();

        assert!(tid1 < tid2, "Later TIDs should sort after earlier ones");
    }

    #[test]
    fn test_parse_tid_timestamp() {
        let tid = generate_tid();
        let timestamp = parse_tid_timestamp(&tid);
        assert!(timestamp.is_some());

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64;
        let parsed = timestamp.unwrap();
        assert!(
            now.abs_diff(parsed) < 1_000_000,
            "Parsed timestamp {} too far from now {}",
            parsed,
            now
        );
    }

    #[test]
    fn test_is_valid_tid() {
        let tid = generate_tid();
        assert!(is_valid_tid(&tid));

        assert!(!is_valid_tid("short"));
        assert!(!is_valid_tid("toolongstring!"));
        assert!(!is_valid_tid("0123456789012"));
    }

    #[test]
    fn test_roundtrip_encoding() {
        let value: u64 = 0x123456789ABCDEF0;
        let encoded = encode_base32_sort(value);
        let decoded = decode_base32_sort(&encoded);
        assert_eq!(decoded, Some(value));
    }
}
