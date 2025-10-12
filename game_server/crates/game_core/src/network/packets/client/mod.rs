use bevy::{log, prelude::*};
use l2r_core::packets::{ClientPacketBuffer, ClientPacketId, L2rClientPacket, L2rSerializeError};
use std::convert::TryFrom;

mod action;
mod attack;
mod auth_login;
mod bypass_command;
mod cannot_move_anymore;
mod char_creation;
mod character_select;
mod double_slash_command;
mod move_backward_to_location;
mod multisell_choose;
mod protocol_verision;
mod request_action_use;
pub mod request_auto_shots;
mod request_destroy_item;
mod request_dispel;
mod request_drop_item;
mod request_magic_skill_use;
mod request_restart_point;
mod say;
mod shortcut_delete;
mod shortcut_registration;
mod single_slash_command;
mod use_item;
mod validate_position;

pub use action::*;
pub use attack::*;
pub use auth_login::*;
pub use bypass_command::*;
pub use cannot_move_anymore::*;
pub use char_creation::*;
pub use character_select::*;
pub use double_slash_command::*;
pub use move_backward_to_location::*;
pub use multisell_choose::*;
pub use protocol_verision::*;
pub use request_action_use::*;
pub use request_destroy_item::*;
pub use request_dispel::*;
pub use request_drop_item::*;
pub use request_magic_skill_use::*;
pub use request_restart_point::*;
pub use say::*;
pub use shortcut_delete::*;
pub use shortcut_registration::*;
pub use single_slash_command::*;
pub use use_item::*;
pub use validate_position::*;

#[derive(Clone, Debug, Reflect)]
pub enum GameClientPacket {
    Unknown(Vec<u8>),
    Attack(attack::Attack),
    Action(action::Action),
    ProtocolVersion(protocol_verision::ClientProtocolVersion),
    AuthLoginRequest(auth_login::AuthLoginRequest),
    CharacterSelect(character_select::CharacterSelect),
    EnterWorld,
    Say(say::Say),
    ValidatePosition(validate_position::ValidatePosition),
    MoveBackwardToLocation(move_backward_to_location::MoveBackwardToLocation),
    CannotMoveAnymore(cannot_move_anymore::CannotMoveAnymore),
    RequestCharCreateMenu,
    RequestCharCreate(char_creation::RequestCharCreate),
    RequestGotoLobby,
    RequestCharDelete(char_creation::RequestCharDelete),
    RequestDropItem(request_drop_item::RequestDropItem),
    RequestDestroyItem(request_destroy_item::RequestDestroyItem),
    RequestShowMap,
    RequestRestart,
    Appearing,
    RequestLogout,
    RequestManorList,
    RequestKeyMapping,
    RequestCancelTarget,
    SingleSlashCommand(single_slash_command::SingleSlashCommand),
    DoubleSlashCommand(double_slash_command::DoubleSlashCommand),
    RequestActionUse(request_action_use::RequestActionUse),
    RequestItemList,
    UseItem(use_item::UseItem),
    RequestRestartPoint(request_restart_point::RequestRestartPoint),
    NetPing,
    RequestMagicSkillUse(request_magic_skill_use::RequestMagicSkillUse),
    RequestDispel(request_dispel::RequestDispel),
    MultisellChoose(multisell_choose::MultisellChoose),
    BypassCommand(bypass_command::BypassCommand),
    RequestShortcutRegistration(shortcut_registration::RequestShortcutRegistration),
    RequestShortcutDelete(shortcut_delete::RequestShortcutDelete),
    RequestAutoShots(request_auto_shots::RequestAutoShots),
}

pub struct GameClientPacketCodes;

impl GameClientPacketCodes {
    const LOGOUT: ClientPacketId = ClientPacketId::new(0x00);
    const ATTACK: ClientPacketId = ClientPacketId::new(0x01);
    const _REQUEST_START_PLEDGE_WAR: ClientPacketId = ClientPacketId::new(0x03);
    const _REQUEST_REPLY_START_PLEDGE: ClientPacketId = ClientPacketId::new(0x04);
    const _REQUEST_STOP_PLEDGE_WAR: ClientPacketId = ClientPacketId::new(0x05);
    const _REQUEST_REPLY_STOP_PLEDGE_WAR: ClientPacketId = ClientPacketId::new(0x06);
    const _REQUEST_SURRENDER_PLEDGE_WAR: ClientPacketId = ClientPacketId::new(0x07);
    const _REQUEST_REPLY_SURRENDER_PLEDGE_WAR: ClientPacketId = ClientPacketId::new(0x08);
    const _REQUEST_SET_PLEDGE_CREST: ClientPacketId = ClientPacketId::new(0x09);
    const _REQUEST_GIVE_NICK_NAME: ClientPacketId = ClientPacketId::new(0x0B);
    const CHAR_CREATE_REQUEST: ClientPacketId = ClientPacketId::new(0x0C);
    const CHAR_DELETE_REQUEST: ClientPacketId = ClientPacketId::new(0x0D);
    const PROTOCOL_VERSION: ClientPacketId = ClientPacketId::new(0x0E);
    const MOVE_BACKWARD_TO_LOCATION: ClientPacketId = ClientPacketId::new(0x0F);
    const ENTER_WORLD: ClientPacketId = ClientPacketId::new(0x11);
    const CHARACTER_SELECT: ClientPacketId = ClientPacketId::new(0x12);
    const NEW_CHARACTER_MENU: ClientPacketId = ClientPacketId::new(0x13);
    const REQUEST_ITEM_LIST: ClientPacketId = ClientPacketId::new(0x14);
    const _REQUEST_UN_EQUIP_ITEM: ClientPacketId = ClientPacketId::new(0x16);
    const REQUEST_DROP_ITEM: ClientPacketId = ClientPacketId::new(0x17);
    const USE_ITEM: ClientPacketId = ClientPacketId::new(0x19);
    const _TRADE_REQUEST: ClientPacketId = ClientPacketId::new(0x1A);
    const _ADD_TRADE_ITEM: ClientPacketId = ClientPacketId::new(0x1B);
    const _TRADE_DONE: ClientPacketId = ClientPacketId::new(0x1C);
    const ACTION: ClientPacketId = ClientPacketId::new(0x1F);
    const _REQUEST_LINK_HTML: ClientPacketId = ClientPacketId::new(0x22);
    const BYPASS_COMMAND: ClientPacketId = ClientPacketId::new(0x23);
    const _REQUEST_BBS_WRITE: ClientPacketId = ClientPacketId::new(0x24);
    const _REQUEST_JOIN_PLEDGE: ClientPacketId = ClientPacketId::new(0x26);
    const _REQUEST_ANSWER_JOIN_PLEDGE: ClientPacketId = ClientPacketId::new(0x27);
    const _REQUEST_WITHDRAWAL_PLEDGE: ClientPacketId = ClientPacketId::new(0x28);
    const _REQUEST_OUST_PLEDGE_MEMBER: ClientPacketId = ClientPacketId::new(0x29);
    const AUTH_LOGIN_REQUEST: ClientPacketId = ClientPacketId::new(0x2B);
    const _REQUEST_GET_ITEM_FROM_PET: ClientPacketId = ClientPacketId::new(0x2C);
    const _REQUEST_ALLY_INFO: ClientPacketId = ClientPacketId::new(0x2E);
    const _REQUEST_CRYSTALLIZE_ITEM: ClientPacketId = ClientPacketId::new(0x2F);
    const _REQUEST_PRIVATE_STORE_MANAGE_SELL: ClientPacketId = ClientPacketId::new(0x30);
    const _SET_PRIVATE_STORE_LIST_SELL: ClientPacketId = ClientPacketId::new(0x31);
    const _ATTACK_REQUEST: ClientPacketId = ClientPacketId::new(0x32);
    const _REQUEST_TELEPORT: ClientPacketId = ClientPacketId::new(0x33);
    const _SOCIAL_ACTION: ClientPacketId = ClientPacketId::new(0x34);
    const _CHANGE_MOVE_TYPE: ClientPacketId = ClientPacketId::new(0x35);
    const _CHANGE_WAIT_TYPE: ClientPacketId = ClientPacketId::new(0x36);
    const _REQUEST_SELL_ITEM: ClientPacketId = ClientPacketId::new(0x37);
    const _REQUEST_MAGIC_SKILL_LIST: ClientPacketId = ClientPacketId::new(0x38);
    const REQUEST_MAGIC_SKILL_USE: ClientPacketId = ClientPacketId::new(0x39);
    const APPEARING: ClientPacketId = ClientPacketId::new(0x3A);
    const _SEND_WARE_HOUSE_DEPOSIT_LIST: ClientPacketId = ClientPacketId::new(0x3B);
    const _SEND_WARE_HOUSE_WITH_DRAW_LIST: ClientPacketId = ClientPacketId::new(0x3C);
    const REQUEST_SHORT_CUT_REG: ClientPacketId = ClientPacketId::new(0x3D);
    const REQUEST_SHORT_CUT_DEL: ClientPacketId = ClientPacketId::new(0x3F);
    const _REQUEST_BUY_ITEM: ClientPacketId = ClientPacketId::new(0x40);
    const _REQUEST_JOIN_PARTY: ClientPacketId = ClientPacketId::new(0x42);
    const _REQUEST_ANSWER_JOIN_PARTY: ClientPacketId = ClientPacketId::new(0x43);
    const _REQUEST_WITH_DRAWAL_PARTY: ClientPacketId = ClientPacketId::new(0x44);
    const _REQUEST_OUST_PARTY_MEMBER: ClientPacketId = ClientPacketId::new(0x45);
    const CANNOT_MOVE_ANYMORE: ClientPacketId = ClientPacketId::new(0x47);
    const REQUEST_CANCEL_TARGET: ClientPacketId = ClientPacketId::new(0x48);
    const SAY: ClientPacketId = ClientPacketId::new(0x49);
    const _REQUEST_PLEDGE_MEMBER_LIST: ClientPacketId = ClientPacketId::new(0x4D);
    const _REQUEST_MAGIC_LIST: ClientPacketId = ClientPacketId::new(0x4F);
    const _REQUEST_SKILL_LIST: ClientPacketId = ClientPacketId::new(0x50);
    const _MOVE_WITH_DELTA: ClientPacketId = ClientPacketId::new(0x52);
    const _REQUEST_GET_ON_VEHICLE: ClientPacketId = ClientPacketId::new(0x53);
    const _REQUEST_GET_OFF_VEHICLE: ClientPacketId = ClientPacketId::new(0x54);
    const _ANSWER_TRADE_REQUEST: ClientPacketId = ClientPacketId::new(0x55);
    const REQUEST_ACTION_USE: ClientPacketId = ClientPacketId::new(0x56);
    const REQUEST_RESTART: ClientPacketId = ClientPacketId::new(0x57);
    const VALIDATE_POSITION: ClientPacketId = ClientPacketId::new(0x59);
    const _START_ROTATING: ClientPacketId = ClientPacketId::new(0x5B);
    const _FINISH_ROTATING: ClientPacketId = ClientPacketId::new(0x5C);
    const _REQUEST_SHOW_BOARD: ClientPacketId = ClientPacketId::new(0x5E);
    const _REQUEST_ENCHANT_ITEM: ClientPacketId = ClientPacketId::new(0x5F);
    const REQUEST_DESTROY_ITEM: ClientPacketId = ClientPacketId::new(0x60);
    const _REQUEST_QUEST_LIST: ClientPacketId = ClientPacketId::new(0x62);
    const _REQUEST_QUEST_ABORT: ClientPacketId = ClientPacketId::new(0x63);
    const _REQUEST_PLEDGE_INFO: ClientPacketId = ClientPacketId::new(0x65);
    const _REQUEST_PLEDGE_EXTENDED_INFO: ClientPacketId = ClientPacketId::new(0x66);
    const _REQUEST_PLEDGE_CREST: ClientPacketId = ClientPacketId::new(0x67);
    const _REQUEST_SEND_FRIEND_MSG: ClientPacketId = ClientPacketId::new(0x6B);
    const REQUEST_SHOW_MAP: ClientPacketId = ClientPacketId::new(0x6C);
    const _REQUEST_RECORD_INFO: ClientPacketId = ClientPacketId::new(0x6E);
    const _REQUEST_HENNA_EQUIP: ClientPacketId = ClientPacketId::new(0x6F);
    const _REQUEST_HENNA_REMOVE_LIST: ClientPacketId = ClientPacketId::new(0x70);
    const _REQUEST_HENNA_ITEM_REMOVE_INFO: ClientPacketId = ClientPacketId::new(0x71);
    const _REQUEST_HENNA_REMOVE: ClientPacketId = ClientPacketId::new(0x72);
    const _REQUEST_ACQUIRE_SKILL_INFO: ClientPacketId = ClientPacketId::new(0x73);
    const DOUBLE_SLASH_COMMAND: ClientPacketId = ClientPacketId::new(0x74);
    const _REQUEST_MOVE_TO_LOCATION_IN_VEHICLE: ClientPacketId = ClientPacketId::new(0x75);
    const _CANNOT_MOVE_ANYMORE_IN_VEHICLE: ClientPacketId = ClientPacketId::new(0x76);
    const _REQUEST_FRIEND_INVITE: ClientPacketId = ClientPacketId::new(0x77);
    const _REQUEST_ANSWER_FRIEND_INVITE: ClientPacketId = ClientPacketId::new(0x78);
    const _REQUEST_FRIEND_LIST: ClientPacketId = ClientPacketId::new(0x79);
    const _REQUEST_FRIEND_DEL: ClientPacketId = ClientPacketId::new(0x7A);
    const _CHARACTER_RESTORE: ClientPacketId = ClientPacketId::new(0x7B);
    const _REQUEST_ACQUIRE_SKILL: ClientPacketId = ClientPacketId::new(0x7C);
    const REQUEST_RESTART_POINT: ClientPacketId = ClientPacketId::new(0x7D);
    const _REQUEST_GM_COMMAND: ClientPacketId = ClientPacketId::new(0x7E);
    const _REQUEST_PARTY_MATCH_CONFIG: ClientPacketId = ClientPacketId::new(0x7F);
    const _REQUEST_PARTY_MATCH_LIST: ClientPacketId = ClientPacketId::new(0x80);
    const _REQUEST_PARTY_MATCH_DETAIL: ClientPacketId = ClientPacketId::new(0x81);
    const _REQUEST_PRIVATE_STORE_BUY: ClientPacketId = ClientPacketId::new(0x83);
    const _REQUEST_TUTORIAL_LINK_HTML: ClientPacketId = ClientPacketId::new(0x85);
    const _REQUEST_TUTORIAL_PASS_CMD_TO_SERVER: ClientPacketId = ClientPacketId::new(0x86);
    const _REQUEST_TUTORIAL_QUESTION_MARK: ClientPacketId = ClientPacketId::new(0x87);
    const _REQUEST_TUTORIAL_CLIENT_EVENT: ClientPacketId = ClientPacketId::new(0x88);
    const _REQUEST_PETITION: ClientPacketId = ClientPacketId::new(0x89);
    const _REQUEST_PETITION_CANCEL: ClientPacketId = ClientPacketId::new(0x8A);
    const _REQUEST_GM_LIST: ClientPacketId = ClientPacketId::new(0x8B);
    const _REQUEST_JOIN_ALLY: ClientPacketId = ClientPacketId::new(0x8C);
    const _REQUEST_ANSWER_JOIN_ALLY: ClientPacketId = ClientPacketId::new(0x8D);
    const _ALLY_LEAVE: ClientPacketId = ClientPacketId::new(0x8E);
    const _ALLY_DISMISS: ClientPacketId = ClientPacketId::new(0x8F);
    const _REQUEST_DISMISS_ALLY: ClientPacketId = ClientPacketId::new(0x90);
    const _REQUEST_SET_ALLY_CREST: ClientPacketId = ClientPacketId::new(0x91);
    const _REQUEST_ALLY_CREST: ClientPacketId = ClientPacketId::new(0x92);
    const _REQUEST_CHANGE_PET_NAME: ClientPacketId = ClientPacketId::new(0x93);
    const _REQUEST_PET_USE_ITEM: ClientPacketId = ClientPacketId::new(0x94);
    const _REQUEST_GIVE_ITEM_TO_PET: ClientPacketId = ClientPacketId::new(0x95);
    const _REQUEST_PRIVATE_STORE_QUIT_SELL: ClientPacketId = ClientPacketId::new(0x96);
    const _SET_PRIVATE_STORE_MSG_SELL: ClientPacketId = ClientPacketId::new(0x97);
    const _REQUEST_PET_GET_ITEM: ClientPacketId = ClientPacketId::new(0x98);
    const _REQUEST_PRIVATE_STORE_MANAGE_BUY: ClientPacketId = ClientPacketId::new(0x99);
    const _SET_PRIVATE_STORE_LIST_BUY: ClientPacketId = ClientPacketId::new(0x9A);
    const _REQUEST_PRIVATE_STORE_QUIT_BUY: ClientPacketId = ClientPacketId::new(0x9C);
    const _SET_PRIVATE_STORE_MSG_BUY: ClientPacketId = ClientPacketId::new(0x9D);
    const _REQUEST_PRIVATE_STORE_SELL: ClientPacketId = ClientPacketId::new(0x9F);
    const _SEND_TIME_CHECK_PACKET: ClientPacketId = ClientPacketId::new(0xA0);
    const _REQUEST_SKILL_COOL_TIME: ClientPacketId = ClientPacketId::new(0xA6);
    const _REQUEST_PACKAGE_SENDABLE_ITEM_LIST: ClientPacketId = ClientPacketId::new(0xA7);
    const _REQUEST_PACKAGE_SEND: ClientPacketId = ClientPacketId::new(0xA8);
    const _REQUEST_BLOCK: ClientPacketId = ClientPacketId::new(0xA9);
    const _REQUEST_SIEGE_INFO: ClientPacketId = ClientPacketId::new(0xAA);
    const _REQUEST_SIEGE_ATTACKER_LIST: ClientPacketId = ClientPacketId::new(0xAB);
    const _REQUEST_SIEGE_DEFENDER_LIST: ClientPacketId = ClientPacketId::new(0xAC);
    const _REQUEST_JOIN_SIEGE: ClientPacketId = ClientPacketId::new(0xAD);
    const _REQUEST_CONFIRM_SIEGE_WAITING_LIST: ClientPacketId = ClientPacketId::new(0xAE);
    const _REQUEST_SET_CASTLE_SIEGE_TIME: ClientPacketId = ClientPacketId::new(0xAF);
    const MULTI_SELL_CHOOSE: ClientPacketId = ClientPacketId::new(0xB0);
    const NET_PING: ClientPacketId = ClientPacketId::new(0xB1);
    const _REQUEST_REMAIN_TIME: ClientPacketId = ClientPacketId::new(0xB2);
    const SINGLE_SLASH_COMMAND: ClientPacketId = ClientPacketId::new(0xB3);
    const _SNOOP_QUIT: ClientPacketId = ClientPacketId::new(0xB4);
    const _REQUEST_RECIPE_BOOK_OPEN: ClientPacketId = ClientPacketId::new(0xB5);
    const _REQUEST_RECIPE_BOOK_DESTROY: ClientPacketId = ClientPacketId::new(0xB6);
    const _REQUEST_RECIPE_ITEM_MAKE_INFO: ClientPacketId = ClientPacketId::new(0xB7);
    const _REQUEST_RECIPE_ITEM_MAKE_SELF: ClientPacketId = ClientPacketId::new(0xB8);
    const _REQUEST_RECIPE_SHOP_MANAGE_LIST: ClientPacketId = ClientPacketId::new(0xB9);
    const _REQUEST_RECIPE_SHOP_MESSAGE_SET: ClientPacketId = ClientPacketId::new(0xBA);
    const _REQUEST_RECIPE_SHOP_LIST_SET: ClientPacketId = ClientPacketId::new(0xBB);
    const _REQUEST_RECIPE_SHOP_MANAGE_QUIT: ClientPacketId = ClientPacketId::new(0xBC);
    const _REQUEST_RECIPE_SHOP_MANAGE_CANCEL: ClientPacketId = ClientPacketId::new(0xBD);
    const _REQUEST_RECIPE_SHOP_MAKE_INFO: ClientPacketId = ClientPacketId::new(0xBE);
    const _REQUEST_RECIPE_SHOP_MAKE_ITEM: ClientPacketId = ClientPacketId::new(0xBF);
    const _REQUEST_RECIPE_SHOP_MANAGE_PREV: ClientPacketId = ClientPacketId::new(0xC0);
    const _OBSERVER_RETURN: ClientPacketId = ClientPacketId::new(0xC1);
    const _REQUEST_EVALUATE: ClientPacketId = ClientPacketId::new(0xC2);
    const _REQUEST_HENNA_ITEM_LIST: ClientPacketId = ClientPacketId::new(0xC3);
    const _REQUEST_HENNA_ITEM_INFO: ClientPacketId = ClientPacketId::new(0xC4);
    const _REQUEST_BUY_SEED: ClientPacketId = ClientPacketId::new(0xC5);
    const _DLG_ANSWER: ClientPacketId = ClientPacketId::new(0xC6);
    const _REQUEST_PREVIEW_ITEM: ClientPacketId = ClientPacketId::new(0xC7);
    const _REQUEST_SSQ_STATUS: ClientPacketId = ClientPacketId::new(0xC8);
    const _REQUEST_PETITION_FEEDBACK: ClientPacketId = ClientPacketId::new(0xC9);
    const _GAME_GUARD_REPLY: ClientPacketId = ClientPacketId::new(0xCB);
    const _REQUEST_PLEDGE_POWER: ClientPacketId = ClientPacketId::new(0xCC);
    const _REQUEST_MAKE_MACRO: ClientPacketId = ClientPacketId::new(0xCD);
    const _REQUEST_DELETE_MACRO: ClientPacketId = ClientPacketId::new(0xCE);
    const _REQUEST_BUY_PROCURE: ClientPacketId = ClientPacketId::new(0xCF);
    // Ex-packets
    const REQUEST_GO_TO_LOBBY: ClientPacketId = ClientPacketId::new_ex(0x36);
    const REQUEST_MANOR_LIST: ClientPacketId = ClientPacketId::new_ex(0x01);
    const REQUEST_KEY_MAPPING: ClientPacketId = ClientPacketId::new_ex(0x21);
    const REQUEST_DISPEL: ClientPacketId = ClientPacketId::new_ex(0x4B);
    const REQUEST_AUTO_SOULSHOT: ClientPacketId = ClientPacketId::new_ex(0x0D);
}

impl TryFrom<ClientPacketBuffer> for GameClientPacket {
    type Error = L2rSerializeError;

    fn try_from(mut buffer: ClientPacketBuffer) -> Result<Self, Self::Error> {
        if buffer.is_empty() {
            return Err(L2rSerializeError::NotEnoughBytes);
        }
        match ClientPacketId::from(&mut buffer) {
            GameClientPacketCodes::ATTACK => Ok(Self::Attack(attack::Attack::try_from(buffer)?)),
            GameClientPacketCodes::PROTOCOL_VERSION => Ok(Self::ProtocolVersion(
                protocol_verision::ClientProtocolVersion::try_from(buffer)?,
            )),
            GameClientPacketCodes::AUTH_LOGIN_REQUEST => Ok(Self::AuthLoginRequest(
                auth_login::AuthLoginRequest::try_from(buffer)?,
            )),
            GameClientPacketCodes::ENTER_WORLD => Ok(Self::EnterWorld),
            GameClientPacketCodes::CHARACTER_SELECT => Ok(Self::CharacterSelect(
                character_select::CharacterSelect::try_from(buffer)?,
            )),
            GameClientPacketCodes::SAY => Ok(Self::Say(say::Say::try_from(buffer)?)),
            GameClientPacketCodes::MOVE_BACKWARD_TO_LOCATION => Ok(Self::MoveBackwardToLocation(
                move_backward_to_location::MoveBackwardToLocation::try_from(buffer)?,
            )),
            GameClientPacketCodes::VALIDATE_POSITION => Ok(Self::ValidatePosition(
                validate_position::ValidatePosition::try_from(buffer)?,
            )),
            GameClientPacketCodes::CANNOT_MOVE_ANYMORE => Ok(Self::CannotMoveAnymore(
                cannot_move_anymore::CannotMoveAnymore::try_from(buffer)?,
            )),
            GameClientPacketCodes::NEW_CHARACTER_MENU => Ok(Self::RequestCharCreateMenu),
            GameClientPacketCodes::CHAR_CREATE_REQUEST => Ok(Self::RequestCharCreate(
                char_creation::RequestCharCreate::try_from(buffer)?,
            )),
            GameClientPacketCodes::CHAR_DELETE_REQUEST => Ok(Self::RequestCharDelete(
                char_creation::RequestCharDelete::try_from(buffer)?,
            )),
            GameClientPacketCodes::REQUEST_GO_TO_LOBBY => Ok(Self::RequestGotoLobby),
            GameClientPacketCodes::REQUEST_SHOW_MAP => Ok(Self::RequestShowMap),
            GameClientPacketCodes::REQUEST_RESTART => Ok(Self::RequestRestart),
            GameClientPacketCodes::LOGOUT => Ok(Self::RequestLogout),
            GameClientPacketCodes::REQUEST_MANOR_LIST => Ok(Self::RequestManorList),
            GameClientPacketCodes::REQUEST_KEY_MAPPING => Ok(Self::RequestKeyMapping),
            GameClientPacketCodes::ACTION => Ok(Self::Action(action::Action::try_from(buffer)?)),
            GameClientPacketCodes::REQUEST_CANCEL_TARGET => Ok(Self::RequestCancelTarget),
            GameClientPacketCodes::SINGLE_SLASH_COMMAND => Ok(Self::SingleSlashCommand(
                single_slash_command::SingleSlashCommand::try_from(buffer)?,
            )),
            GameClientPacketCodes::DOUBLE_SLASH_COMMAND => Ok(Self::DoubleSlashCommand(
                double_slash_command::DoubleSlashCommand::try_from(buffer)?,
            )),
            GameClientPacketCodes::REQUEST_ACTION_USE => Ok(Self::RequestActionUse(
                request_action_use::RequestActionUse::try_from(buffer)?,
            )),
            GameClientPacketCodes::REQUEST_ITEM_LIST => Ok(Self::RequestItemList),
            GameClientPacketCodes::USE_ITEM => {
                Ok(Self::UseItem(use_item::UseItem::try_from(buffer)?))
            }
            GameClientPacketCodes::REQUEST_RESTART_POINT => Ok(Self::RequestRestartPoint(
                request_restart_point::RequestRestartPoint::try_from(buffer)?,
            )),
            GameClientPacketCodes::NET_PING => Ok(Self::NetPing),
            GameClientPacketCodes::REQUEST_MAGIC_SKILL_USE => Ok(Self::RequestMagicSkillUse(
                request_magic_skill_use::RequestMagicSkillUse::try_from(buffer)?,
            )),
            GameClientPacketCodes::REQUEST_DROP_ITEM => Ok(Self::RequestDropItem(
                request_drop_item::RequestDropItem::try_from(buffer)?,
            )),
            GameClientPacketCodes::REQUEST_DESTROY_ITEM => Ok(Self::RequestDestroyItem(
                request_destroy_item::RequestDestroyItem::try_from(buffer)?,
            )),
            GameClientPacketCodes::BYPASS_COMMAND => Ok(Self::BypassCommand(
                bypass_command::BypassCommand::try_from(buffer)?,
            )),
            GameClientPacketCodes::REQUEST_SHORT_CUT_REG => Ok(Self::RequestShortcutRegistration(
                shortcut_registration::RequestShortcutRegistration::try_from(buffer)?,
            )),
            GameClientPacketCodes::REQUEST_SHORT_CUT_DEL => Ok(Self::RequestShortcutDelete(
                shortcut_delete::RequestShortcutDelete::try_from(buffer)?,
            )),
            GameClientPacketCodes::REQUEST_DISPEL => Ok(Self::RequestDispel(
                request_dispel::RequestDispel::try_from(buffer)?,
            )),
            GameClientPacketCodes::MULTI_SELL_CHOOSE => Ok(Self::MultisellChoose(
                multisell_choose::MultisellChoose::try_from(buffer)?,
            )),
            GameClientPacketCodes::APPEARING => Ok(Self::Appearing),
            GameClientPacketCodes::REQUEST_AUTO_SOULSHOT => Ok(Self::RequestAutoShots(
                request_auto_shots::RequestAutoShots::try_from(buffer)?,
            )),
            _ => {
                // Its possible to throw an error here, but we can just log it and return an unknown packet
                // because if we throw an error here, the client will disconnect
                // and i dont want to disconnect the client just because of an unknown packet during development
                let error = L2rSerializeError::new("Unknown packet".to_string(), buffer.as_slice());
                log::warn!("{}", error);
                Ok(GameClientPacket::Unknown(buffer.into()))
            }
        }
    }
}

impl L2rClientPacket for GameClientPacket {}
