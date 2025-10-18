use super::{ClientPacketBuffer, PacketId};
use bevy::prelude::*;
use log;
use smallvec::SmallVec;
use std::fmt::Debug;

const SERVER_BUFFER_CAPACITY: usize = 256;

#[derive(Clone, Debug, Default, Deref, DerefMut, Eq, PartialEq)]
pub struct ServerPacketBuffer(SmallVec<[u8; SERVER_BUFFER_CAPACITY]>);

impl ServerPacketBuffer {
    pub fn new() -> Self {
        Self(SmallVec::with_capacity(SERVER_BUFFER_CAPACITY))
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(SmallVec::with_capacity(capacity))
    }

    pub fn str(&mut self, string: &str) {
        let mut bytes = Vec::with_capacity(string.len() * 2 + 2);
        for char in string.encode_utf16() {
            bytes.extend(char.to_le_bytes());
        }
        bytes.extend(&[0, 0]);
        self.0.extend_from_slice(&bytes);
    }

    pub fn i8(&mut self, value: i8) {
        self.push(value as u8)
    }

    pub fn u8(&mut self, value: u8) {
        self.push(value)
    }

    pub fn u8_from_usize(&mut self, value: usize) {
        self.u8(value as u8)
    }

    pub fn bool(&mut self, value: bool) {
        self.u8(u8::from(value))
    }

    pub fn i16(&mut self, value: i16) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u16(&mut self, value: u16) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u16_from_usize(&mut self, value: usize) {
        self.u16(value as u16)
    }

    pub fn u16_from_bool(&mut self, value: bool) {
        self.u16(u16::from(value))
    }

    pub fn i32(&mut self, value: i32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u32(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn u32_from_usize(&mut self, value: usize) {
        self.u32(value as u32)
    }

    pub fn u32_from_bool(&mut self, value: bool) {
        self.u32(u32::from(value))
    }

    pub fn u64(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn i64(&mut self, value: i64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn f32(&mut self, value: f32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn f64(&mut self, value: f64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    /// Log buffer utilization statistics for optimization purposes
    pub fn log_utilization(&self) {
        let len = self.0.len();
        let capacity = self.0.capacity();

        if len < SERVER_BUFFER_CAPACITY / 4 {
            log::trace!(
                "ServerPacketBuffer underutilized: {} bytes used of {} capacity ({}% utilization)",
                len,
                capacity,
                (len * 100) / capacity.max(1)
            );
        } else if len > SERVER_BUFFER_CAPACITY && capacity > len * 2 {
            log::trace!(
                "ServerPacketBuffer over-allocated: {} bytes used of {} capacity ({}% utilization)",
                len,
                capacity,
                (len * 100) / capacity
            );
        }
    }
}

impl From<Vec<u8>> for ServerPacketBuffer {
    fn from(vec: Vec<u8>) -> Self {
        if vec.len() > SERVER_BUFFER_CAPACITY {
            log::debug!(
                "ServerPacketBuffer created from large Vec ({} bytes) > SERVER_BUFFER_CAPACITY ({}), will heap allocate",
                vec.len(),
                SERVER_BUFFER_CAPACITY
            );
        } else if vec.capacity() > SERVER_BUFFER_CAPACITY * 2 {
            log::debug!(
                "ServerPacketBuffer created from over-allocated Vec (capacity: {}, len: {}), wasted memory detected",
                vec.capacity(),
                vec.len()
            );
        }
        Self(SmallVec::from_vec(vec))
    }
}

impl From<ServerPacketBuffer> for Vec<u8> {
    fn from(buffer: ServerPacketBuffer) -> Self {
        buffer.0.into_vec()
    }
}

impl From<SmallVec<[u8; SERVER_BUFFER_CAPACITY]>> for ServerPacketBuffer {
    fn from(smallvec: SmallVec<[u8; SERVER_BUFFER_CAPACITY]>) -> Self {
        Self(smallvec)
    }
}

impl From<ServerPacketBuffer> for SmallVec<[u8; SERVER_BUFFER_CAPACITY]> {
    fn from(buffer: ServerPacketBuffer) -> Self {
        buffer.0
    }
}

impl Iterator for ServerPacketBuffer {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.remove(0))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

impl ExactSizeIterator for ServerPacketBuffer {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> IntoIterator for &'a ServerPacketBuffer {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ClientPacketId(PacketId);
impl ClientPacketId {
    pub const EX_ID: u8 = u8::MAX - 47;
    pub const fn new(id: u8) -> Self {
        ClientPacketId(PacketId::new(id))
    }
    pub const fn new_ex(ex_id: u16) -> Self {
        ClientPacketId(PacketId::new_ex(Self::EX_ID, ex_id))
    }
    pub fn to_le_bytes(self) -> Vec<u8> {
        self.0.to_le_bytes()
    }
}

impl From<&mut ClientPacketBuffer> for ClientPacketId {
    fn from(buffer: &mut ClientPacketBuffer) -> Self {
        let bytes = buffer.as_slice();
        if bytes.len() > 1 {
            if bytes[0] >= ClientPacketId::EX_ID {
                let id = ClientPacketId::new_ex(u16::from_le_bytes([bytes[1], bytes[2]]));
                buffer.position(3);
                id
            } else {
                let id = ClientPacketId::new(bytes[0]);
                buffer.position(1);
                id
            }
        } else {
            let id = ClientPacketId::new(bytes[0]);
            buffer.position(1);
            id
        }
    }
}

pub trait L2rServerPackets: Event + Send + Sync + Debug + 'static {
    fn buffer(self) -> ServerPacketBuffer;
}

#[macro_export]
macro_rules! impl_buffer {
    ($type:ty, $($variant:ident),+) => {
        impl L2rServerPackets for $type {
            fn buffer(self) -> ServerPacketBuffer {
                match self {
                    $(Self::$variant(p) => p.buffer(),)+
                }
            }
        }
    }
}

pub trait L2rServerPacket: Send + Sync {
    fn buffer(self) -> ServerPacketBuffer;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerPacketId(PacketId);
impl ServerPacketId {
    pub const EX_ID: u8 = u8::MAX - 1;
    pub const fn new(id: u8) -> Self {
        ServerPacketId(PacketId::new(id))
    }
    pub const fn new_ex(ex_id: u16) -> Self {
        ServerPacketId(PacketId::new_ex(Self::EX_ID, ex_id))
    }
    pub fn to_le_bytes(self) -> Vec<u8> {
        self.0.to_le_bytes()
    }
}

impl From<&SmallVec<[u8; SERVER_BUFFER_CAPACITY]>> for ServerPacketId {
    fn from(value: &SmallVec<[u8; SERVER_BUFFER_CAPACITY]>) -> Self {
        if value.len() > 1 {
            if value[0] >= ServerPacketId::EX_ID {
                ServerPacketId::new_ex(u16::from_le_bytes([value[1], value[2]]))
            } else {
                ServerPacketId::new(value[0])
            }
        } else {
            ServerPacketId::new(value[0])
        }
    }
}

impl From<&Vec<u8>> for ServerPacketId {
    fn from(value: &Vec<u8>) -> Self {
        if value.len() > 1 {
            if value[0] >= ServerPacketId::EX_ID {
                ServerPacketId::new_ex(u16::from_le_bytes([value[1], value[2]]))
            } else {
                ServerPacketId::new(value[0])
            }
        } else {
            ServerPacketId::new(value[0])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let buffer = ServerPacketBuffer::new();
        assert!(buffer.0.is_empty());
        assert!(buffer.capacity() >= 128);
    }

    #[test]
    fn test_with_capacity() {
        let buffer = ServerPacketBuffer::with_capacity(256);
        assert!(buffer.0.is_empty());
        assert!(buffer.capacity() >= 256);
    }

    #[test]
    fn test_str() {
        let mut buffer = ServerPacketBuffer::new();

        let test_str = "Hello";

        buffer.str(test_str);

        // Each character in UTF-16 takes 2 bytes, plus 2 bytes for null terminator
        assert_eq!(buffer.len(), test_str.len() * 2 + 2);

        // Check that the string was correctly encoded
        let bytes: Vec<u8> = buffer.into();
        let expected = vec![
            72, 0, // 'H'
            101, 0, // 'e'
            108, 0, // 'l'
            108, 0, // 'l'
            111, 0, // 'o'
            0, 0, // null terminator
        ];
        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_primitive_types() {
        let mut buffer = ServerPacketBuffer::new();

        buffer.i8(42);
        buffer.u8(255);
        buffer.bool(true);
        buffer.i16(-12345);
        buffer.u16(54321);
        buffer.u16_from_usize(1000);
        buffer.u16_from_bool(true);
        buffer.i32(-1234567);
        buffer.u32(7654321);
        buffer.u32_from_usize(1000000);
        buffer.u32_from_bool(false);
        buffer.u64(0x1234567890ABCDEF);
        buffer.i64(-9223372036854775807);
        buffer.f32(core::f32::consts::PI);
        buffer.f64(core::f64::consts::E);

        // Verify the buffer contains the expected number of bytes
        let expected_size = 1 +  // i8
            1 +  // u8
            1 +  // bool
            2 +  // i16
            2 +  // u16
            2 +  // u16_from_usize
            2 +  // u16_from_bool
            4 +  // i32
            4 +  // u32
            4 +  // u32_from_usize
            4 +  // u32_from_bool
            8 +  // u64
            8 +  // i64
            4 +  // f32
            8; // f64

        assert_eq!(buffer.len(), expected_size);

        // Convert to Vec and check specific values
        let bytes: Vec<u8> = buffer.into();

        // Check first byte (i8 = 42)
        assert_eq!(bytes[0], 42);

        // Check second byte (u8 = 255)
        assert_eq!(bytes[1], 255);

        // Check 5, 6 bytes = u16 = 54321
        assert_eq!(u16::from_le_bytes([bytes[5], bytes[6]]), 54321);
    }

    #[test]
    fn test_to_vec() {
        let mut buffer = ServerPacketBuffer::new();
        buffer.u32(0x12345678);

        let vec: Vec<u8> = buffer.into();
        assert_eq!(vec, [0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_from_into() {
        let data = vec![1, 2, 3, 4];

        // Test From trait
        let buffer: ServerPacketBuffer = data.clone().into();
        assert_eq!(buffer.as_slice(), &[1, 2, 3, 4]);

        // Test Into trait
        let vec: Vec<u8> = buffer.into();
        assert_eq!(vec, data);
    }
}
