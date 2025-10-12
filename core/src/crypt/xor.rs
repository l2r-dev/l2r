use bevy::log;
use rand::Rng;

/// XOR encryption key used for encoding and decoding data.
/// The key is a 4-byte integer, which is generated randomly.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct XorKey(i32);
impl XorKey {
    /// Creates a new XOR key with a random value.
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self(rng.gen_range(0..i32::MAX))
    }

    pub fn to_le_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

impl From<&[u8]> for XorKey {
    fn from(bytes: &[u8]) -> Self {
        Self(i32::from_le_bytes(bytes.try_into().unwrap_or_default()))
    }
}

impl From<usize> for XorKey {
    fn from(value: usize) -> Self {
        Self(value as i32)
    }
}

impl Default for XorKey {
    fn default() -> Self {
        Self::new()
    }
}

/// Encryption and decryption engine using XOR encryption.
/// The data is expected to be in the format of 4-byte chunks, with the first and last 4-byte chunks being the key.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct XorCrypt {
    key: XorKey,
}

impl XorCrypt {
    pub fn new(key: XorKey) -> Self {
        Self { key }
    }

    /// Encodes the data using XOR encryption. Modifies the input data in place.
    /// Returns the final key that should be appended to the data.
    pub fn encode(&self, data: &mut [u8]) -> [u8; 4] {
        let key = self.key.to_le_bytes();
        let size = data.len() / 4;
        let mut temp_key = i32::from_le_bytes(key);
        log::trace!("First key is: {:?}", temp_key);

        // Split the data into chunks of 4 bytes, but not 1st 4 bytes and last 4 bytes
        data.chunks_exact_mut(4).skip(1).take(size).for_each(|src| {
            let mut bytes = [0u8; 4];
            log::trace!("src: {:?}", src);
            bytes.copy_from_slice(src);
            let mut chunk = i32::from_le_bytes(bytes);
            temp_key = temp_key.wrapping_add(chunk);
            chunk ^= temp_key;
            src.copy_from_slice(&chunk.to_le_bytes());
            log::trace!("encrypted: {:?}, key: {:?}", chunk.to_le_bytes(), temp_key);
        });
        temp_key.to_le_bytes()
    }

    /// Decodes the data using XOR encryption. Modifies the input data in place.
    pub fn decode(data: &mut [u8]) {
        if data.len() < 4 {
            return;
        }

        let size = data.len() / 4 - 1;

        // Get last XOR key from the data and use it as the initial XOR key
        let last_key_bytes: [u8; 4] = data[data.len() - 4..data.len()]
            .try_into()
            .unwrap_or_default();
        let mut temp_key = i32::from_le_bytes(last_key_bytes);

        // Splits the data into chunks of 4 bytes, but not 1st 4 bytes and last 4 bytes
        data.chunks_exact_mut(4).rev().take(size).for_each(|src| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(src);
            let mut chunk = i32::from_le_bytes(bytes);
            chunk ^= temp_key;
            temp_key = temp_key.wrapping_sub(chunk);
            src.copy_from_slice(&chunk.to_le_bytes());
        });
    }
}
