/// Checksum used to verify the integrity of LoginServer packets.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckSum(i32);
impl CheckSum {
    pub fn new(chksum: i32) -> Self {
        Self(chksum)
    }

    pub fn to_le_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

impl From<&[u8]> for CheckSum {
    fn from(bytes: &[u8]) -> Self {
        let chksum = bytes.chunks_exact(4).fold(0i32, |acc, chunk| {
            acc ^ i32::from_le_bytes(chunk.try_into().unwrap_or_default())
        });
        CheckSum::new(chksum)
    }
}
