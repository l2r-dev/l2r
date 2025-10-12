use super::LoginServerPacketCode;
use l2r_core::packets::{L2rServerPacket, ServerPacketBuffer};

#[derive(Clone, Debug, Default)]
#[repr(u8)]
pub enum PlayFail {
    #[default]
    ReasonNoMessage = 0x00,
    ReasonSystemErrorLoginLater = 0x01,
    ReasonUserOrPassWrong = 0x02,
    ReasonAccessFailedTryAgainLater = 0x04,
    ReasonAccountInfoIncorrectContactSupport = 0x05,
    ReasonAccountInUse = 0x07,
    ReasonUnder18YearsKR = 0x0C,
    ReasonServerOverloaded = 0x0F,
    ReasonServerMaintenance = 0x10,
    ReasonTempPassExpired = 0x11,
    ReasonGameTimeExpired = 0x12,
    ReasonNoTimeLeft = 0x13,
    ReasonSystemError = 0x14,
    ReasonAccessFailed = 0x15,
    ReasonRestrictedIP = 0x16,
    ReasonWeekUsageFinished = 0x1E,
    ReasonSecurityCardNumberInvalid = 0x1F,
    ReasonAgeNotVerifiedCantLogBetween10PM6AM = 0x20,
    ReasonServerCannotBeAccessedByYourCoupon = 0x21,
    ReasonDualBox = 0x23,
    ReasonInactive = 0x24,
    ReasonUserAgreementRejectedOnWebsite = 0x25,
    ReasonGuardianConsentRequired = 0x26,
    ReasonUserAgreementDeclinedOrWithdrawlRequest = 0x27,
    ReasonAccountSuspendedCall = 0x28,
    ReasonChangePasswordAndQuizOnWebsite = 0x29,
    ReasonAlreadyLoggedInTo10Accounts = 0x2A,
    ReasonMasterAccountRestricted = 0x2B,
    ReasonCertificationFailed = 0x2E,
    ReasonTelephoneCertificationUnavailable = 0x2F,
    ReasonTelephoneSignalsDelayed = 0x30,
    ReasonCertificationFailedLineBusy = 0x31,
    ReasonCertificationServiceNumberExpiredOrIncorrect = 0x32,
    ReasonCertificationServiceCurrentlyBeingChecked = 0x33,
    ReasonCertificationServiceCantBeUsedHeavyVolume = 0x34,
    ReasonCertificationServiceExpiredGameplayBlocked = 0x35,
    ReasonCertificationFailed3TimesGameplayBlocked30Min = 0x36,
    ReasonCertificationDailyUseExceeded = 0x37,
    ReasonCertificationUnderwayTryAgainLater = 0x38,
}

impl From<PlayFail> for u8 {
    fn from(reason: PlayFail) -> u8 {
        reason as u8
    }
}

impl L2rServerPacket for PlayFail {
    fn buffer(self) -> ServerPacketBuffer {
        let mut buffer = ServerPacketBuffer::new();
        buffer.extend(LoginServerPacketCode::PLAY_FAIL.to_le_bytes());
        buffer.u8(self.into());
        buffer
    }
}
