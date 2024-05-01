use hex;
use rand::{seq::SliceRandom, thread_rng};
use sha2::{Digest, Sha256};

pub fn generate_random_name() -> String {
    let mut rng = thread_rng();
    let adjective = adjectives().choose(&mut rng).unwrap();
    // let adjective = adjectives().sample(&mut rng).unwrap();
    let noun1 = nouns1().choose(&mut rng).unwrap();
    let noun2 = nouns2().choose(&mut rng).unwrap();

    format!("{adjective}-{noun1}-{noun2}")
}

fn adjectives() -> &'static [&'static str] {
    static ADJECTIVES: &[&str] = &["funny", "smart", "creative", "friendly", "great"];
    ADJECTIVES
}

fn nouns1() -> &'static [&'static str] {
    static NOUNS1: &[&str] = &["dog", "cat", "flower", "tree", "house"];
    NOUNS1
}

fn nouns2() -> &'static [&'static str] {
    static NOUNS2: &[&str] = &["cookie", "cake", "frosting"];
    NOUNS2
}

pub fn hash_random_name(name: String) -> String {
    let hashed_name = Sha256::digest(name.as_bytes());
    hex::encode(hashed_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_name() {
        let name = generate_random_name();

        assert!(name.contains('-'));
        assert!(name.split('-').count() == 3);
        assert!(name.len() > 0);
    }
}
