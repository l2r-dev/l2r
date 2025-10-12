use num_enum::IntoPrimitive;
use rand::Rng;
use std::{hash::Hash, net::Ipv4Addr};

mod names;
mod server_table;

pub use names::*;
pub use server_table::*;

#[derive(Clone, Copy, Eq, Hash, IntoPrimitive, PartialEq)]
#[repr(u8)]
pub enum Status {
    Auto,
    Good,
    Normal,
    Full,
    Down,
    GMOnly,
}

#[derive(Clone, Copy, Eq, Hash, IntoPrimitive, PartialEq)]
#[repr(u32)]
pub enum ServerType {
    Normal = 0x01,
    Relax = 0x02,
    Test = 0x04,
    NoLabel = 0x08,
    CreationRestricted = 0x10,
    Event = 0x20,
    Free = 0x40,
}

#[derive(Clone, Copy, Eq, Hash, IntoPrimitive, PartialEq)]
#[repr(u8)]
pub enum AgeLimit {
    NoLimit = 0,
    Older15 = 15,
    Older18 = 18,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct GameServer {
    id: i32,
    is_authed: bool,
    status: Option<Status>,
    addrs: Vec<Ipv4Addr>,
    port: u32,
    is_pvp: bool,
    server_type: ServerType,
    age_limit: AgeLimit,
    is_showing_brackets: bool,
    max_players: i32,
}

impl GameServer {
    pub fn new(
        id: i32,
        is_authed: bool,
        addrs: Vec<Ipv4Addr>,
        port: u32,
        server_type: ServerType,
        is_showing_brackets: bool,
        max_players: i32,
    ) -> Self {
        Self {
            id,
            is_authed,
            status: None,
            addrs,
            port,
            is_pvp: true,
            server_type,
            age_limit: AgeLimit::NoLimit,
            is_showing_brackets,
            max_players,
        }
    }
    pub fn is_pvp(&self) -> u8 {
        self.is_pvp as u8
    }
    pub fn server_type(&self) -> ServerType {
        self.server_type
    }
    pub fn port(&self) -> u32 {
        self.port
    }
    pub fn current_player_count(&self) -> u16 {
        let mut rng = rand::thread_rng();
        let count = rng.gen_range(1..3000);
        count as u16
    }
    pub fn max_players(&self) -> u16 {
        self.max_players as u16
    }
    pub fn is_showing_brackets(&self) -> bool {
        self.is_showing_brackets
    }
    pub fn id(&self) -> u8 {
        self.id as u8
    }
    pub fn age_limit(&self) -> AgeLimit {
        self.age_limit
    }
    pub fn status(&self) -> Status {
        // TODO: implement getting proper status
        Status::Good
    }
    // TODO: return server address based on client's ip
    pub fn server_address(&self) -> Ipv4Addr {
        self.addrs[0]
    }
}
