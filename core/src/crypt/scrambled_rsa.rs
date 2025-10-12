use num_traits::{FromPrimitive, Num};
use rsa::{BigUint, RsaPrivateKey, RsaPublicKey, traits::PublicKeyParts};

#[derive(Clone)]
pub struct ScrambledRsaKey {
    pub private_key: RsaPrivateKey,
}

impl ScrambledRsaKey {
    pub fn new() -> Self {
        ScrambledRsaKey {
            private_key: RsaPrivateKey::new(&mut rand::thread_rng(), 1024)
                .expect("failed to generate a key"),
        }
    }
    // create not-random static key for tests
    pub fn fixed() -> Self {
        let private_key = RsaPrivateKey::from_components(
            BigUint::from_str_radix("9353930466774385905609975137998169297361893554149986716853295022578535724979677252958524466350471210367835187480748268864277464700638583474144061408845077", 10).unwrap(),
            BigUint::from_u64(65537).unwrap_or_default(),
            BigUint::from_str_radix("7266398431328116344057699379749222532279343923819063639497049039389899328538543087657733766554155839834519529439851673014800261285757759040931985506583861", 10).unwrap(),
            vec![
                BigUint::from_str_radix("98920366548084643601728869055592650835572950932266967461790948584315647051443",10).unwrap(),
                BigUint::from_str_radix("94560208308847015747498523884063394671606671904944666360068158221458669711639", 10).unwrap()
            ],
        ).unwrap();
        ScrambledRsaKey { private_key }
    }

    pub fn new_vec(n: usize) -> Vec<Self> {
        let mut vec = Vec::with_capacity(n);
        for _ in 0..n {
            vec.push(ScrambledRsaKey::new());
        }
        vec
    }
    pub fn _public_key(&self) -> RsaPublicKey {
        RsaPublicKey::from(&self.private_key)
    }
    pub fn scramble(&self) -> [u8; 128] {
        let mut bytes = self.private_key.to_public_key().n().to_bytes_be();
        bytes.resize(128, 0);

        // Step 1: Swap first 4 bytes with bytes at 77 (0x4D)
        let (left, right) = bytes.split_at_mut(77);
        let temp = left[0..4].to_vec();
        left[0..4].copy_from_slice(&right[0..4]);
        right[0..4].copy_from_slice(&temp);

        // Step 2: XOR first 64 bytes with last 64 bytes (0x40)
        let (first_half, second_half) = bytes.split_at_mut(64);
        for (a, b) in first_half.iter_mut().zip(second_half.iter()) {
            *a ^= *b;
        }

        // Step 3: XOR bytes 13-16 with bytes 52-56 (0x0D-0x10 and 0x34-0x38)
        let (prefix, rest) = bytes.split_at_mut(13);
        let (a_slice, rest) = rest.split_at_mut(4); // a_slice: bytes[13..13+4]
        let offset = 52 - (prefix.len() + a_slice.len()); // Compute the offset to reach 52 from current position
        let (_middle, rest) = rest.split_at_mut(offset);
        let (b_slice, _) = rest.split_at_mut(4); // b_slice: bytes[52..52+4]

        for (a, b) in a_slice.iter_mut().zip(b_slice.iter()) {
            *a ^= *b;
        }

        // Step 4: XOR last 64 bytes with first 64 bytes
        let (first_half, second_half) = bytes.split_at_mut(64);
        for (a, b) in second_half.iter_mut().zip(first_half.iter()) {
            *a ^= *b;
        }

        let mut scrambled_bytes = [0u8; 128];
        scrambled_bytes.copy_from_slice(&bytes);
        scrambled_bytes
    }

    pub fn _unscramble(bytes: [u8; 128]) -> BigUint {
        let mut bytes = bytes;

        // Reverse Step 4: XOR last 64 bytes with first 64 bytes
        let (first_half, second_half) = bytes.split_at_mut(64);
        for (a, b) in second_half.iter_mut().zip(first_half.iter()) {
            *a ^= *b;
        }

        // Reverse Step 3: XOR bytes 13-16 with bytes 52-56
        let (prefix, rest) = bytes.split_at_mut(13);
        let (a_slice, rest) = rest.split_at_mut(4); // a_slice: bytes[13..13+4]
        let offset = 52 - (prefix.len() + a_slice.len()); // Compute the offset
        let (_middle, rest) = rest.split_at_mut(offset);
        let (b_slice, _) = rest.split_at_mut(4); // b_slice: bytes[52..52+4]

        for (a, b) in a_slice.iter_mut().zip(b_slice.iter()) {
            *a ^= *b;
        }

        // Reverse Step 2: XOR first 64 bytes with last 64 bytes
        let (first_half, second_half) = bytes.split_at_mut(64);
        for (a, b) in first_half.iter_mut().zip(second_half.iter()) {
            *a ^= *b;
        }

        // Reverse Step 1: Swap first 4 bytes with bytes at 77
        let (left, right) = bytes.split_at_mut(77);
        let temp = left[0..4].to_vec();
        left[0..4].copy_from_slice(&right[0..4]);
        right[0..4].copy_from_slice(&temp);

        BigUint::from_bytes_le(&bytes)
    }
}
impl Default for ScrambledRsaKey {
    fn default() -> Self {
        ScrambledRsaKey::new()
    }
}
