use super::L2rSerializeError;
use bevy::{log, prelude::*};
use smallvec::SmallVec;
use std::fmt::Debug;

pub type Result<T = ()> = std::result::Result<T, L2rSerializeError>;

const CLIENT_BUFFER_CAPACITY: usize = 64;

#[derive(Clone, Default, Deref, DerefMut)]
pub struct ClientPacketBuffer {
    #[deref]
    data: SmallVec<[u8; CLIENT_BUFFER_CAPACITY]>,
    position: usize,
}

impl Debug for ClientPacketBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = crate::utils::format_bytes_hex(self.data.as_slice());
        f.debug_struct("ClientPacketBuffer")
            .field("position", &self.position)
            .field("data", &bytes)
            .finish()
    }
}

impl ClientPacketBuffer {
    pub fn new(bytes: &[u8]) -> Self {
        if CLIENT_BUFFER_CAPACITY > bytes.len() {
            log::trace!(
                "CLIENT_BUFFER_CAPACITY ({}) > buffer size ({}), potential optimization available",
                CLIENT_BUFFER_CAPACITY,
                bytes.len()
            );
        }
        Self {
            data: SmallVec::from_slice(bytes),
            position: 0,
        }
    }

    pub fn position(&mut self, pos: usize) -> &mut Self {
        if pos <= self.data.len() {
            self.position = pos;
        }
        self
    }

    pub fn skip(&mut self, count: usize) -> Result<&mut Self> {
        if self.position + count > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        self.position += count;
        Ok(self)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn bytes(&mut self, count: usize) -> Result<&[u8]> {
        if self.position + count > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let bytes = &self.data[self.position..self.position + count];
        self.position += count;
        Ok(bytes)
    }

    pub fn bytes_at(&self, start: usize, count: usize) -> Result<&[u8]> {
        if start + count > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        Ok(&self.data[start..start + count])
    }

    pub fn str(&mut self) -> Result<String> {
        let mut len = 0;
        while self.position + len * 2 + 1 < self.data.len() {
            let chunk = &self.data[self.position + len * 2..self.position + len * 2 + 2];
            let char = u16::from_le_bytes([chunk[0], chunk[1]]);
            if char == 0 {
                break;
            }
            len += 1;
        }

        let utf16: Vec<u16> = self.data[self.position..self.position + len * 2]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        self.position += len * 2 + 2;

        String::from_utf16(&utf16).map_err(L2rSerializeError::from)
    }

    pub fn i8(&mut self) -> Result<i8> {
        if self.position >= self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let value = self.data[self.position] as i8;
        self.position += 1;
        Ok(value)
    }

    pub fn u8(&mut self) -> Result<u8> {
        if self.position >= self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    pub fn bool(&mut self) -> Result<bool> {
        self.u8().map(|v| v != 0)
    }

    pub fn bool_from_u32(&mut self) -> Result<bool> {
        self.u32().map(|v| v != 0)
    }

    pub fn i16(&mut self) -> Result<i16> {
        let bytes = self.data[self.position..self.position + 2].try_into()?;
        let value = i16::from_le_bytes(bytes);
        self.position += 2;
        Ok(value)
    }

    pub fn u16(&mut self) -> Result<u16> {
        let bytes = self.data[self.position..self.position + 2].try_into()?;
        let value = u16::from_le_bytes(bytes);
        self.position += 2;
        Ok(value)
    }

    pub fn i32(&mut self) -> Result<i32> {
        let bytes = self.data[self.position..self.position + 4].try_into()?;
        let value = i32::from_le_bytes(bytes);
        self.position += 4;
        Ok(value)
    }

    pub fn u32(&mut self) -> Result<u32> {
        if self.position + 4 > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let bytes = self.data[self.position..self.position + 4].try_into()?;
        let value = u32::from_le_bytes(bytes);
        self.position += 4;
        Ok(value)
    }

    pub fn u32_at(&self, pos: usize) -> Result<u32> {
        if pos + 4 > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let bytes = self.data[pos..pos + 4].try_into()?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn i32_at(&self, pos: usize) -> Result<i32> {
        if pos + 4 > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let bytes = self.data[pos..pos + 4].try_into()?;
        Ok(i32::from_le_bytes(bytes))
    }

    pub fn u64(&mut self) -> Result<u64> {
        if self.position + 8 > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let bytes = &self.data[self.position..self.position + 8];
        let value = u64::from_le_bytes(bytes.try_into()?);
        self.position += 8;
        Ok(value)
    }

    pub fn i64(&mut self) -> Result<i64> {
        if self.position + 8 > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }

        let bytes = &self.data[self.position..self.position + 8];
        let value = i64::from_le_bytes(bytes.try_into()?);
        self.position += 8;

        Ok(value)
    }

    pub fn f32(&mut self) -> Result<f32> {
        if self.position + 4 > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let bytes = self.data[self.position..self.position + 4].try_into()?;
        let value = f32::from_le_bytes(bytes);
        self.position += 4;
        Ok(value)
    }

    pub fn f64(&mut self) -> Result<f64> {
        if self.position + 8 > self.data.len() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        let bytes = self.data[self.position..self.position + 8].try_into()?;
        let value = f64::from_le_bytes(bytes);
        self.position += 8;
        Ok(value)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn remaining_bytes(&self) -> &[u8] {
        &self.data[self.position..]
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.position
    }
}

impl From<Vec<u8>> for ClientPacketBuffer {
    fn from(vec: Vec<u8>) -> Self {
        Self {
            data: SmallVec::from_vec(vec),
            position: 0,
        }
    }
}

impl From<ClientPacketBuffer> for Vec<u8> {
    fn from(buffer: ClientPacketBuffer) -> Self {
        buffer.data.into_vec()
    }
}

impl From<SmallVec<[u8; CLIENT_BUFFER_CAPACITY]>> for ClientPacketBuffer {
    fn from(smallvec: SmallVec<[u8; CLIENT_BUFFER_CAPACITY]>) -> Self {
        Self {
            data: smallvec,
            position: 0,
        }
    }
}

impl From<ClientPacketBuffer> for SmallVec<[u8; CLIENT_BUFFER_CAPACITY]> {
    fn from(buffer: ClientPacketBuffer) -> Self {
        buffer.data
    }
}

pub trait L2rClientPacket:
    TryFrom<ClientPacketBuffer> + Sized + Send + Sync + Reflect + Debug + 'static
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let data = [1, 2, 3, 4];
        let buffer = ClientPacketBuffer::new(&data);

        assert_eq!(buffer.as_slice(), &data);
        assert_eq!(buffer.get_position(), 0);
    }

    #[test]
    fn test_position() {
        let data = [1, 2, 3, 4];
        let mut buffer = ClientPacketBuffer::new(&data);

        buffer.position(2);
        assert_eq!(buffer.get_position(), 2);

        // Position should not exceed buffer size
        buffer.position(10);
        assert_eq!(buffer.get_position(), 2);
    }

    #[test]
    fn test_skip() {
        let data = [1, 2, 3, 4, 5];
        let mut buffer = ClientPacketBuffer::new(&data);

        let result = buffer.skip(2);
        assert!(result.is_ok());
        assert_eq!(buffer.get_position(), 2);

        // Skip beyond buffer size
        let result = buffer.skip(4);
        assert!(result.is_err());
        assert_eq!(buffer.get_position(), 2);
    }

    #[test]
    fn test_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut buffer = ClientPacketBuffer::new(&data);

        let bytes = buffer.bytes(3).unwrap();
        assert_eq!(bytes, &[1, 2, 3]);
        assert_eq!(buffer.get_position(), 3);

        // Try to read beyond buffer size
        let result = buffer.bytes(3);
        assert!(result.is_err());
    }

    #[test]
    fn test_bytes_at() {
        let data = [1, 2, 3, 4, 5];
        let buffer = ClientPacketBuffer::new(&data);

        let bytes = buffer.bytes_at(1, 3).unwrap();
        assert_eq!(bytes, &[2, 3, 4]);

        // Try to read beyond buffer size
        let result = buffer.bytes_at(3, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_str() {
        // "Test" encoded as UTF-16LE + null terminator
        let data = [84, 0, 101, 0, 115, 0, 116, 0, 0, 0];
        let mut buffer = ClientPacketBuffer::new(&data);

        let s = buffer.str().unwrap();
        assert_eq!(s, "Test");
        assert_eq!(buffer.get_position(), 10);
    }

    #[test]
    fn test_primitive_types() {
        let mut data = vec![];

        data.push(i8::MAX as u8);
        data.push(u8::MAX);

        data.extend_from_slice(&i16::MAX.to_le_bytes());
        data.extend_from_slice(&u16::MAX.to_le_bytes());

        data.extend_from_slice(&i32::MAX.to_le_bytes());
        data.extend_from_slice(&u32::MAX.to_le_bytes());

        data.extend_from_slice(&i64::MAX.to_le_bytes());
        data.extend_from_slice(&u64::MAX.to_le_bytes());

        data.extend_from_slice(&core::f32::consts::PI.to_le_bytes());
        data.extend_from_slice(&core::f64::consts::E.to_le_bytes());

        data.push(1); // true
        data.extend_from_slice(&1_u32.to_le_bytes()); // true as u32

        let mut buffer = ClientPacketBuffer::new(&data);

        assert_eq!(buffer.i8().unwrap(), i8::MAX);
        assert_eq!(buffer.u8().unwrap(), u8::MAX);
        assert_eq!(buffer.i16().unwrap(), i16::MAX);
        assert_eq!(buffer.u16().unwrap(), u16::MAX);
        assert_eq!(buffer.i32().unwrap(), i32::MAX);
        assert_eq!(buffer.u32().unwrap(), u32::MAX);
        assert_eq!(buffer.i64().unwrap(), i64::MAX);
        assert_eq!(buffer.u64().unwrap(), u64::MAX);

        let f32_val = buffer.f32().unwrap();
        assert!((f32_val - core::f32::consts::PI).abs() < 0.00001);

        let f64_val = buffer.f64().unwrap();
        assert!((f64_val - core::f64::consts::E).abs() < 0.00001);

        assert!(buffer.bool().unwrap());
        assert!(buffer.bool_from_u32().unwrap());
    }

    #[test]
    fn test_u32_at_and_i32_at() {
        let data = [1, 0, 0, 0, 255, 255, 255, 255];
        let buffer = ClientPacketBuffer::new(&data);

        assert_eq!(buffer.u32_at(0).unwrap(), 1);
        assert_eq!(buffer.i32_at(4).unwrap(), -1);

        // Try to read beyond buffer size
        let result = buffer.u32_at(5);
        assert!(result.is_err());
    }

    #[test]
    fn test_remaining() {
        let data = [1, 2, 3, 4, 5];
        let mut buffer = ClientPacketBuffer::new(&data);

        assert_eq!(buffer.remaining(), 5);

        buffer.skip(3).unwrap();
        assert_eq!(buffer.remaining(), 2);
        assert_eq!(buffer.remaining_bytes(), &[4, 5]);
    }

    #[test]
    fn test_from_conversions() {
        let data = vec![1, 2, 3];

        let buffer = ClientPacketBuffer::from(data.clone());
        assert_eq!(buffer.as_slice(), &[1, 2, 3]);

        let vec: Vec<u8> = buffer.into();
        assert_eq!(vec, data);
    }
}
