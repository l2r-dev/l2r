use crate::crypt::{blowfish::BlowfishKey, scrambled_rsa::ScrambledRsaKey, xor::XorKey};
use rsa::traits::PublicKeyParts;

#[test]
fn xor_key_from_slice() {
    let key = [1u8, 2, 3, 4];
    let xor_key = XorKey::from(&key[..]);
    assert_eq!(xor_key.to_le_bytes(), key);
}

#[test]
fn test_blowfish_new_with_provided_key() {
    let key = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let blowfish_key = BlowfishKey::from(key);
    assert_eq!(key, blowfish_key.to_le_bytes());
}

#[test]
fn test_blowfish_new_with_unique_random_key() {
    let blowfish_key1 = BlowfishKey::new();
    let blowfish_key2 = BlowfishKey::new();
    let key1 = blowfish_key1.to_le_bytes();
    let key2 = blowfish_key2.to_le_bytes();

    assert_ne!(key1, key2, "Generated keys should be unique");
}

#[test]
fn test_blowfish_as_bytes() {
    let key = [
        42u8, 0, 255, 128, 64, 32, 16, 8, 4, 2, 1, 127, 254, 129, 65, 33,
    ];
    let blowfish_key = BlowfishKey::from(key);
    assert_eq!(key, blowfish_key.to_le_bytes());
}

#[test]
fn test_key_initialization() {
    let vec = ScrambledRsaKey::new_vec(2);
    assert_eq!(vec.len(), 2, "Vector should contain 5 keys");
}

#[test]
fn test_public_key_derivation() {
    let key = ScrambledRsaKey::new();
    let public_key = key._public_key();
    assert_eq!(
        public_key.n().to_bytes_le().len(),
        key.private_key.to_public_key().n().to_bytes_le().len(),
        "Public key length should match the derived one from private key"
    );
}

#[test]
fn test_scramble_unscramble_round_trip() {
    let key = ScrambledRsaKey::new();
    let original_data = key.private_key.to_public_key().n().to_bytes_be();
    let scrambled = key.scramble();
    let unscrambled = ScrambledRsaKey::_unscramble(scrambled).to_bytes_le();
    assert_eq!(
        unscrambled, original_data,
        "Unscrambled data should match the original data"
    );
}
