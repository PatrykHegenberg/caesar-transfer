use hex;
use rand::{seq::SliceRandom, thread_rng};
use sha2::{Digest, Sha256};


/// Generates a random name composed of an adjective, a noun, and another noun.
///
/// # Returns
///
/// A string in the format "{adjective}-{noun1}-{noun2}".
pub fn generate_random_name() -> String {
    let mut rng = thread_rng();
    let adjective = adjectives().choose(&mut rng).unwrap();
    let noun1 = nouns1().choose(&mut rng).unwrap();
    let noun2 = nouns2().choose(&mut rng).unwrap();

    format!("{adjective}-{noun1}-{noun2}")
}

/// Returns a random adjective.
///
/// # Returns
///
/// A `&'static str` representing an adjective.
fn adjectives() -> &'static [&'static str] {
    // Define a list of adjectives.
    static ADJECTIVES: &[&str] = &["funny", "smart", "creative", "friendly", "great"];
    ADJECTIVES
}

/// Returns a random noun.
///
/// # Returns
///
/// A `&'static str` representing a noun.
fn nouns1() -> &'static [&'static str] {
    // Define a list of nouns.
    static NOUNS1: &[&str] = &["dog", "cat", "flower", "tree", "house"];
    NOUNS1
}

/// Returns a random noun.
///
/// # Returns
///
/// A `&'static str` representing a noun.
fn nouns2() -> &'static [&'static str] {
    // Define a list of nouns.
    static NOUNS2: &[&str] = &["cookie", "cake", "frosting"];
    NOUNS2
}

/// Hashes a given name using SHA256 and returns the hex-encoded result.
///
/// # Parameters
///
/// * `name`: A `String` representing the name to be hashed.
///
/// # Returns
///
/// A `String` containing the hex-encoded hash of the name.
pub fn hash_random_name(name: String) -> String {
    let hashed_name = Sha256::digest(name.as_bytes());
    hex::encode(hashed_name)
}

/// Replaces occurrences of "ws://" and "wss://" in a given address with "http://" and "https://" respectively.
///
/// # Parameters
///
/// * `address`: A `&str` representing the address to modify.
///
/// # Returns
///
/// A `String` representing the modified address.
pub fn replace_protocol(address: &str) -> String {
    let mut result = address.to_string();
    result = result.replace("ws://", "http://");
    result = result.replace("wss://", "https://");

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_name() {
        let name = generate_random_name();

        assert!(name.contains('-'));
        assert!(name.split('-').count() == 3);
        assert!(name.is_empty());
    }
}
