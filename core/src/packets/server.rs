use super::{ClientPacketBuffer, PacketId};
use bevy::prelude::*;
use smallvec::SmallVec;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use strum::{Display, EnumDiscriminants};

#[derive(Clone, Debug, EnumDiscriminants, Eq, PartialEq)]
#[repr(u8)]
#[strum_discriminants(name(ServerPacketBufferVariant))]
#[strum_discriminants(derive(Display))]
pub enum ServerPacketBuffer {
    Size1(SmallVec<[u8; Self::CAPACITY_1]>),
    Size3(SmallVec<[u8; Self::CAPACITY_3]>),
    Size6(SmallVec<[u8; Self::CAPACITY_6]>),
    Size16(SmallVec<[u8; Self::CAPACITY_16]>),
    Size32(SmallVec<[u8; Self::CAPACITY_32]>),
    Size64(SmallVec<[u8; Self::CAPACITY_64]>),
    Size128(SmallVec<[u8; Self::CAPACITY_128]>),
    Size256(SmallVec<[u8; Self::CAPACITY_256]>),
    Size512(SmallVec<[u8; Self::CAPACITY_512]>),
    Size1024(SmallVec<[u8; Self::CAPACITY_1024]>),
    Size2048(SmallVec<[u8; Self::CAPACITY_2048]>),
    Size4096(SmallVec<[u8; Self::CAPACITY_4096]>),
}

impl ServerPacketBuffer {
    pub const CAPACITY_1: usize = 1;
    pub const CAPACITY_3: usize = 3;
    pub const CAPACITY_6: usize = 6;
    pub const CAPACITY_16: usize = 16;
    pub const CAPACITY_32: usize = 32;
    pub const CAPACITY_64: usize = 64;
    pub const CAPACITY_128: usize = 128;
    pub const CAPACITY_256: usize = 256;
    pub const CAPACITY_512: usize = 512;
    pub const CAPACITY_1024: usize = 1024;
    pub const CAPACITY_2048: usize = 2048;
    pub const CAPACITY_4096: usize = 4096;

    const fn fixed_capacity(&self) -> usize {
        match self {
            Self::Size1(_) => Self::CAPACITY_1,
            Self::Size3(_) => Self::CAPACITY_3,
            Self::Size6(_) => Self::CAPACITY_6,
            Self::Size16(_) => Self::CAPACITY_16,
            Self::Size32(_) => Self::CAPACITY_32,
            Self::Size64(_) => Self::CAPACITY_64,
            Self::Size128(_) => Self::CAPACITY_128,
            Self::Size256(_) => Self::CAPACITY_256,
            Self::Size512(_) => Self::CAPACITY_512,
            Self::Size1024(_) => Self::CAPACITY_1024,
            Self::Size2048(_) => Self::CAPACITY_2048,
            Self::Size4096(_) => Self::CAPACITY_4096,
        }
    }
}

impl Default for ServerPacketBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerPacketBuffer {
    pub fn new() -> Self {
        Self::Size128(SmallVec::new())
    }

    pub fn new_1() -> Self {
        Self::Size1(SmallVec::new())
    }

    pub fn new_3() -> Self {
        Self::Size3(SmallVec::new())
    }

    pub fn new_6() -> Self {
        Self::Size6(SmallVec::new())
    }

    pub fn new_16() -> Self {
        Self::Size16(SmallVec::new())
    }

    pub fn new_32() -> Self {
        Self::Size32(SmallVec::new())
    }

    pub fn new_64() -> Self {
        Self::Size64(SmallVec::new())
    }

    pub fn new_128() -> Self {
        Self::Size128(SmallVec::new())
    }

    pub fn new_256() -> Self {
        Self::Size256(SmallVec::new())
    }

    pub fn new_512() -> Self {
        Self::Size512(SmallVec::new())
    }

    pub fn new_1024() -> Self {
        Self::Size1024(SmallVec::new())
    }

    pub fn new_2048() -> Self {
        Self::Size2048(SmallVec::new())
    }

    pub fn new_4096() -> Self {
        Self::Size4096(SmallVec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= Self::CAPACITY_1 {
            Self::Size1(SmallVec::new())
        } else if capacity <= Self::CAPACITY_3 {
            Self::Size3(SmallVec::new())
        } else if capacity <= Self::CAPACITY_6 {
            Self::Size6(SmallVec::new())
        } else if capacity <= Self::CAPACITY_16 {
            Self::Size16(SmallVec::new())
        } else if capacity <= Self::CAPACITY_32 {
            Self::Size32(SmallVec::new())
        } else if capacity <= Self::CAPACITY_64 {
            Self::Size64(SmallVec::new())
        } else if capacity <= Self::CAPACITY_128 {
            Self::Size128(SmallVec::new())
        } else if capacity <= Self::CAPACITY_256 {
            Self::Size256(SmallVec::new())
        } else if capacity <= Self::CAPACITY_512 {
            Self::Size512(SmallVec::new())
        } else if capacity <= Self::CAPACITY_1024 {
            Self::Size1024(SmallVec::new())
        } else if capacity <= Self::CAPACITY_2048 {
            Self::Size2048(SmallVec::new())
        } else {
            Self::Size4096(SmallVec::new())
        }
    }

    fn push(&mut self, byte: u8) {
        match self {
            Self::Size1(buf) => buf.push(byte),
            Self::Size3(buf) => buf.push(byte),
            Self::Size6(buf) => buf.push(byte),
            Self::Size16(buf) => buf.push(byte),
            Self::Size32(buf) => buf.push(byte),
            Self::Size64(buf) => buf.push(byte),
            Self::Size128(buf) => buf.push(byte),
            Self::Size256(buf) => buf.push(byte),
            Self::Size512(buf) => buf.push(byte),
            Self::Size1024(buf) => buf.push(byte),
            Self::Size2048(buf) => buf.push(byte),
            Self::Size4096(buf) => buf.push(byte),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Size1(buf) => buf.len(),
            Self::Size3(buf) => buf.len(),
            Self::Size6(buf) => buf.len(),
            Self::Size16(buf) => buf.len(),
            Self::Size32(buf) => buf.len(),
            Self::Size64(buf) => buf.len(),
            Self::Size128(buf) => buf.len(),
            Self::Size256(buf) => buf.len(),
            Self::Size512(buf) => buf.len(),
            Self::Size1024(buf) => buf.len(),
            Self::Size2048(buf) => buf.len(),
            Self::Size4096(buf) => buf.len(),
        }
    }

    fn capacity(&self) -> usize {
        match self {
            Self::Size1(buf) => buf.capacity(),
            Self::Size3(buf) => buf.capacity(),
            Self::Size6(buf) => buf.capacity(),
            Self::Size16(buf) => buf.capacity(),
            Self::Size32(buf) => buf.capacity(),
            Self::Size64(buf) => buf.capacity(),
            Self::Size128(buf) => buf.capacity(),
            Self::Size256(buf) => buf.capacity(),
            Self::Size512(buf) => buf.capacity(),
            Self::Size1024(buf) => buf.capacity(),
            Self::Size2048(buf) => buf.capacity(),
            Self::Size4096(buf) => buf.capacity(),
        }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            Self::Size1(buf) => buf.as_mut_slice(),
            Self::Size3(buf) => buf.as_mut_slice(),
            Self::Size6(buf) => buf.as_mut_slice(),
            Self::Size16(buf) => buf.as_mut_slice(),
            Self::Size32(buf) => buf.as_mut_slice(),
            Self::Size64(buf) => buf.as_mut_slice(),
            Self::Size128(buf) => buf.as_mut_slice(),
            Self::Size256(buf) => buf.as_mut_slice(),
            Self::Size512(buf) => buf.as_mut_slice(),
            Self::Size1024(buf) => buf.as_mut_slice(),
            Self::Size2048(buf) => buf.as_mut_slice(),
            Self::Size4096(buf) => buf.as_mut_slice(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Size1(buf) => buf.is_empty(),
            Self::Size3(buf) => buf.is_empty(),
            Self::Size6(buf) => buf.is_empty(),
            Self::Size16(buf) => buf.is_empty(),
            Self::Size32(buf) => buf.is_empty(),
            Self::Size64(buf) => buf.is_empty(),
            Self::Size128(buf) => buf.is_empty(),
            Self::Size256(buf) => buf.is_empty(),
            Self::Size512(buf) => buf.is_empty(),
            Self::Size1024(buf) => buf.is_empty(),
            Self::Size2048(buf) => buf.is_empty(),
            Self::Size4096(buf) => buf.is_empty(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Size1(buf) => buf.as_slice(),
            Self::Size3(buf) => buf.as_slice(),
            Self::Size6(buf) => buf.as_slice(),
            Self::Size16(buf) => buf.as_slice(),
            Self::Size32(buf) => buf.as_slice(),
            Self::Size64(buf) => buf.as_slice(),
            Self::Size128(buf) => buf.as_slice(),
            Self::Size256(buf) => buf.as_slice(),
            Self::Size512(buf) => buf.as_slice(),
            Self::Size1024(buf) => buf.as_slice(),
            Self::Size2048(buf) => buf.as_slice(),
            Self::Size4096(buf) => buf.as_slice(),
        }
    }

    pub fn extend<I: IntoIterator<Item = u8>>(&mut self, iterable: I) {
        match self {
            Self::Size1(buf) => buf.extend(iterable),
            Self::Size3(buf) => buf.extend(iterable),
            Self::Size6(buf) => buf.extend(iterable),
            Self::Size16(buf) => buf.extend(iterable),
            Self::Size32(buf) => buf.extend(iterable),
            Self::Size64(buf) => buf.extend(iterable),
            Self::Size128(buf) => buf.extend(iterable),
            Self::Size256(buf) => buf.extend(iterable),
            Self::Size512(buf) => buf.extend(iterable),
            Self::Size1024(buf) => buf.extend(iterable),
            Self::Size2048(buf) => buf.extend(iterable),
            Self::Size4096(buf) => buf.extend(iterable),
        }
    }

    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> u8,
    {
        match self {
            Self::Size1(buf) => buf.resize_with(new_len, f),
            Self::Size3(buf) => buf.resize_with(new_len, f),
            Self::Size6(buf) => buf.resize_with(new_len, f),
            Self::Size16(buf) => buf.resize_with(new_len, f),
            Self::Size32(buf) => buf.resize_with(new_len, f),
            Self::Size64(buf) => buf.resize_with(new_len, f),
            Self::Size128(buf) => buf.resize_with(new_len, f),
            Self::Size256(buf) => buf.resize_with(new_len, f),
            Self::Size512(buf) => buf.resize_with(new_len, f),
            Self::Size1024(buf) => buf.resize_with(new_len, f),
            Self::Size2048(buf) => buf.resize_with(new_len, f),
            Self::Size4096(buf) => buf.resize_with(new_len, f),
        }
    }

    pub fn truncate(&mut self, len: usize) {
        match self {
            Self::Size1(buf) => buf.truncate(len),
            Self::Size3(buf) => buf.truncate(len),
            Self::Size6(buf) => buf.truncate(len),
            Self::Size16(buf) => buf.truncate(len),
            Self::Size32(buf) => buf.truncate(len),
            Self::Size64(buf) => buf.truncate(len),
            Self::Size128(buf) => buf.truncate(len),
            Self::Size256(buf) => buf.truncate(len),
            Self::Size512(buf) => buf.truncate(len),
            Self::Size1024(buf) => buf.truncate(len),
            Self::Size2048(buf) => buf.truncate(len),
            Self::Size4096(buf) => buf.truncate(len),
        }
    }

    pub fn str(&mut self, string: &str) {
        let mut bytes = Vec::with_capacity(string.len() * 2 + 2);
        for char in string.encode_utf16() {
            bytes.extend(char.to_le_bytes());
        }
        bytes.extend(&[0, 0]);
        self.extend(bytes);
    }

    pub fn i8(&mut self, value: i8) {
        self.push(value as u8);
    }

    pub fn u8(&mut self, value: u8) {
        self.push(value);
    }

    pub fn u8_from_usize(&mut self, value: usize) {
        self.u8(value as u8)
    }

    pub fn bool(&mut self, value: bool) {
        self.u8(u8::from(value))
    }

    pub fn i16(&mut self, value: i16) {
        self.extend(value.to_le_bytes());
    }

    pub fn u16(&mut self, value: u16) {
        self.extend(value.to_le_bytes());
    }

    pub fn u16_from_usize(&mut self, value: usize) {
        self.u16(value as u16)
    }

    pub fn u16_from_bool(&mut self, value: bool) {
        self.u16(u16::from(value))
    }

    pub fn i32(&mut self, value: i32) {
        self.extend(value.to_le_bytes());
    }

    pub fn u32(&mut self, value: u32) {
        self.extend(value.to_le_bytes());
    }

    pub fn u32_from_usize(&mut self, value: usize) {
        self.u32(value as u32)
    }

    pub fn u32_from_bool(&mut self, value: bool) {
        self.u32(u32::from(value))
    }

    pub fn u64(&mut self, value: u64) {
        self.extend(value.to_le_bytes());
    }

    pub fn i64(&mut self, value: i64) {
        self.extend(value.to_le_bytes());
    }

    pub fn f32(&mut self, value: f32) {
        self.extend(value.to_le_bytes());
    }

    pub fn f64(&mut self, value: f64) {
        self.extend(value.to_le_bytes());
    }

    /// Log buffer utilization statistics for optimization purposes
    pub fn log_utilization(&self) {
        let variant = ServerPacketBufferVariant::from(self);
        if variant == ServerPacketBufferVariant::Size1 {
            return;
        }

        let len = self.len();
        let capacity = self.capacity();
        let fixed_capacity = self.fixed_capacity();

        let utilization_pct = if capacity > 0 {
            (len * 100) / capacity
        } else {
            0
        };

        if capacity > fixed_capacity {
            trace!(
                "ServerPacketBuffer::{} heap allocation: {} bytes used of {} capacity ({}% utilization)",
                variant, len, capacity, utilization_pct
            );
        } else if utilization_pct < 51 {
            trace!(
                "ServerPacketBuffer::{} underutilized: {} bytes used of {} capacity ({}% utilization)",
                variant, len, fixed_capacity, utilization_pct
            );
        }
    }
}

impl Deref for ServerPacketBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for ServerPacketBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl From<&[u8]> for ServerPacketBuffer {
    fn from(slice: &[u8]) -> Self {
        let len = slice.len();
        if len <= Self::CAPACITY_1 {
            Self::Size1(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_3 {
            Self::Size3(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_6 {
            Self::Size6(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_16 {
            Self::Size16(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_32 {
            Self::Size32(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_64 {
            Self::Size64(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_128 {
            Self::Size128(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_256 {
            Self::Size256(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_512 {
            Self::Size512(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_1024 {
            Self::Size1024(SmallVec::from_slice(slice))
        } else if len <= Self::CAPACITY_2048 {
            Self::Size2048(SmallVec::from_slice(slice))
        } else {
            Self::Size4096(SmallVec::from_slice(slice))
        }
    }
}

impl From<Vec<u8>> for ServerPacketBuffer {
    fn from(vec: Vec<u8>) -> Self {
        let len = vec.len();
        if len <= Self::CAPACITY_1 {
            Self::Size1(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_3 {
            Self::Size3(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_6 {
            Self::Size6(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_16 {
            Self::Size16(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_32 {
            Self::Size32(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_64 {
            Self::Size64(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_128 {
            Self::Size128(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_256 {
            Self::Size256(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_512 {
            Self::Size512(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_1024 {
            Self::Size1024(SmallVec::from_vec(vec))
        } else if len <= Self::CAPACITY_2048 {
            Self::Size2048(SmallVec::from_vec(vec))
        } else {
            Self::Size4096(SmallVec::from_vec(vec))
        }
    }
}

impl From<ServerPacketBuffer> for Vec<u8> {
    fn from(buffer: ServerPacketBuffer) -> Self {
        match buffer {
            ServerPacketBuffer::Size1(buf) => buf.into_vec(),
            ServerPacketBuffer::Size3(buf) => buf.into_vec(),
            ServerPacketBuffer::Size6(buf) => buf.into_vec(),
            ServerPacketBuffer::Size16(buf) => buf.into_vec(),
            ServerPacketBuffer::Size32(buf) => buf.into_vec(),
            ServerPacketBuffer::Size64(buf) => buf.into_vec(),
            ServerPacketBuffer::Size128(buf) => buf.into_vec(),
            ServerPacketBuffer::Size256(buf) => buf.into_vec(),
            ServerPacketBuffer::Size512(buf) => buf.into_vec(),
            ServerPacketBuffer::Size1024(buf) => buf.into_vec(),
            ServerPacketBuffer::Size2048(buf) => buf.into_vec(),
            ServerPacketBuffer::Size4096(buf) => buf.into_vec(),
        }
    }
}

impl From<SmallVec<[u8; ServerPacketBuffer::CAPACITY_64]>> for ServerPacketBuffer {
    fn from(smallvec: SmallVec<[u8; ServerPacketBuffer::CAPACITY_64]>) -> Self {
        Self::Size64(smallvec)
    }
}

impl Iterator for ServerPacketBuffer {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if ServerPacketBuffer::is_empty(self) {
            None
        } else {
            match self {
                Self::Size1(buf) => Some(buf.remove(0)),
                Self::Size3(buf) => Some(buf.remove(0)),
                Self::Size6(buf) => Some(buf.remove(0)),
                Self::Size16(buf) => Some(buf.remove(0)),
                Self::Size32(buf) => Some(buf.remove(0)),
                Self::Size64(buf) => Some(buf.remove(0)),
                Self::Size128(buf) => Some(buf.remove(0)),
                Self::Size256(buf) => Some(buf.remove(0)),
                Self::Size512(buf) => Some(buf.remove(0)),
                Self::Size1024(buf) => Some(buf.remove(0)),
                Self::Size2048(buf) => Some(buf.remove(0)),
                Self::Size4096(buf) => Some(buf.remove(0)),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for ServerPacketBuffer {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'a> IntoIterator for &'a ServerPacketBuffer {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

#[derive(Clone, Copy, Deref, Eq, PartialEq)]
pub struct ClientPacketId(PacketId);
impl ClientPacketId {
    pub const EX_ID: u8 = u8::MAX - 47;
    pub const fn new(id: u8) -> Self {
        ClientPacketId(PacketId::new(id))
    }
    pub const fn new_ex(ex_id: u16) -> Self {
        ClientPacketId(PacketId::new_ex(Self::EX_ID, ex_id))
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

#[derive(Clone, Copy, Debug, Deref, Eq, PartialEq)]
pub struct ServerPacketId(PacketId);
impl ServerPacketId {
    pub const EX_ID: u8 = u8::MAX - 1;
    pub const fn new(id: u8) -> Self {
        ServerPacketId(PacketId::new(id))
    }
    pub const fn new_ex(ex_id: u16) -> Self {
        ServerPacketId(PacketId::new_ex(Self::EX_ID, ex_id))
    }
}

impl From<&ServerPacketBuffer> for ServerPacketId {
    fn from(buffer: &ServerPacketBuffer) -> Self {
        let slice = buffer.as_slice();
        if slice.len() > 1 {
            if slice[0] >= ServerPacketId::EX_ID {
                ServerPacketId::new_ex(u16::from_le_bytes([slice[1], slice[2]]))
            } else {
                ServerPacketId::new(slice[0])
            }
        } else {
            ServerPacketId::new(slice[0])
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
        assert!(buffer.is_empty());
        assert!(matches!(buffer, ServerPacketBuffer::Size128(_)));
    }

    #[test]
    fn test_with_capacity() {
        let size1 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_1);
        assert!(matches!(size1, ServerPacketBuffer::Size1(_)));

        let size3 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_3);
        assert!(matches!(size3, ServerPacketBuffer::Size3(_)));

        let size6 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_6);
        assert!(matches!(size6, ServerPacketBuffer::Size6(_)));

        let size16 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_16);
        assert!(matches!(size16, ServerPacketBuffer::Size16(_)));

        let size32 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_32);
        assert!(matches!(size32, ServerPacketBuffer::Size32(_)));

        let size64 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_64);
        assert!(matches!(size64, ServerPacketBuffer::Size64(_)));

        let size128 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_128);
        assert!(matches!(size128, ServerPacketBuffer::Size128(_)));

        let size256 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_256);
        assert!(matches!(size256, ServerPacketBuffer::Size256(_)));

        let size512 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_512);
        assert!(matches!(size512, ServerPacketBuffer::Size512(_)));

        let size1024 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_1024);
        assert!(matches!(size1024, ServerPacketBuffer::Size1024(_)));

        let size2048 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_2048);
        assert!(matches!(size2048, ServerPacketBuffer::Size2048(_)));

        let size4096 = ServerPacketBuffer::with_capacity(ServerPacketBuffer::CAPACITY_4096);
        assert!(matches!(size4096, ServerPacketBuffer::Size4096(_)));
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
