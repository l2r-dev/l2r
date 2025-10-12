mod client;
mod serialize;
mod server;

pub use client::*;
pub use serialize::*;
pub use server::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PacketId {
    pub id: u8,
    pub ex_id: Option<u16>,
}
impl PacketId {
    pub const fn new(id: u8) -> Self {
        PacketId { id, ex_id: None }
    }
    pub const fn new_ex(id: u8, ex_id: u16) -> Self {
        PacketId {
            id,
            ex_id: Some(ex_id),
        }
    }
    pub fn to_le_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.id);
        if let Some(value) = self.ex_id {
            bytes.extend(value.to_le_bytes());
        }
        bytes
    }
}
