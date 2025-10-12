use super::LoginServerPacketCode;
use l2r_core::{
    model::session::SessionId,
    packets::{L2rServerPacket, ServerPacketBuffer},
};

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum LoginFailReason {
    NoText,
    SystemErrorLoginLater,
    PasswordDoesNotMatchThisAccount,
    PasswordDoesNotMatchThisAccount2,
    AccessFailedTryLater,
    IncorrectAccountInfoContactCustomerSupport,
    AccessFailedTryLater2,
    AccountAlreadyInUse,
    AccessFailedTryLater3,
    AccessFailedTryLater4,
    AccessFailedTryLater5,
}

impl From<LoginFailReason> for u32 {
    fn from(reason: LoginFailReason) -> u32 {
        reason as u32
    }
}

#[derive(Clone, Debug)]
pub struct LoginFail {
    session_id: SessionId,
    reason: LoginFailReason,
}
impl L2rServerPacket for LoginFail {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(LoginServerPacketCode::LOGIN_FAIL.to_le_bytes());
        buffer.u32(self.reason.into());
        buffer.u32_from_usize(*self.session_id);
        buffer
    }
}
impl LoginFail {
    pub fn new(session_id: SessionId, reason: LoginFailReason) -> Self {
        Self { session_id, reason }
    }
}
