use crate::{
    network::{
        broadcast::BroadcastScope, packets::server::magic_skill_canceled::MagicSkillCanceled,
    },
    skills::SkillList,
};
use bevy::prelude::*;
use derive_more::derive::From;
use l2r_core::packets::{L2rServerPacket, L2rServerPackets, ServerPacketBuffer, ServerPacketId};

mod abnormal_status_update;
mod action_fail;
mod attack;
mod attack_stance_start;
mod attack_stance_stop;
mod change_move_type;
mod change_wait_type;
mod char_create_fail;
mod char_create_success;
mod char_delete_fail;
mod char_delete_success;
mod char_info;
mod char_selection_info;
mod character_selected;
mod creature_say;
mod delete_object;
mod die;
mod door_status_update;
mod drop_item;
mod etc_status_update;
mod ex_basic_action_list;
mod ex_br_extra_user_info;
mod ex_rotation;
mod get_item;
mod inventory_update;
mod item_list;
mod key_packet;
mod logout_ok;
mod magic_skill_canceled;
mod magic_skill_launched;
mod magic_skill_use;
mod move_to_location;
mod move_to_pawn;
mod multisell_list;
mod net_ping;
mod new_character_create_menu;
mod npc_html_message;
mod npc_info;
mod play_sound;
mod response_auto_shots;
mod restart;
mod revive;
mod select_target;
mod setup_gauge;
mod shortcut_init;
mod shortcut_registered;
mod show_map;
mod skill_list;
mod social_action;
mod spawn_item;
mod ssq_info;
mod static_object_info;
mod status_update;
mod stop_move;
mod system_message;
mod target_unselected;
mod teleport_to_location;
mod user_info;
mod validate_location;

pub use abnormal_status_update::*;
pub use action_fail::*;
pub use attack::*;
pub use attack_stance_start::*;
pub use attack_stance_stop::*;
pub use change_move_type::*;
pub use change_wait_type::*;
pub use char_create_fail::*;
pub use char_create_success::*;
pub use char_delete_fail::*;
pub use char_delete_success::*;
pub use char_info::*;
pub use char_selection_info::*;
pub use character_selected::*;
pub use creature_say::*;
pub use delete_object::*;
pub use die::*;
pub use door_status_update::*;
pub use drop_item::*;
pub use etc_status_update::*;
pub use ex_basic_action_list::*;
pub use ex_br_extra_user_info::*;
pub use ex_rotation::*;
pub use get_item::*;
pub use inventory_update::*;
pub use item_list::*;
pub use key_packet::*;
pub use logout_ok::*;
pub use magic_skill_launched::*;
pub use magic_skill_use::*;
pub use move_to_location::*;
pub use move_to_pawn::*;
pub use multisell_list::*;
pub use net_ping::*;
pub use new_character_create_menu::*;
pub use npc_html_message::*;
pub use npc_info::*;
pub use play_sound::*;
pub use response_auto_shots::*;
pub use restart::*;
pub use revive::*;
pub use select_target::*;
pub use setup_gauge::*;
pub use shortcut_init::*;
pub use shortcut_registered::*;
pub use show_map::*;
pub use social_action::*;
pub use spawn_item::*;
pub use ssq_info::*;
pub use static_object_info::*;
pub use status_update::*;
pub use stop_move::*;
use strum::{Display, EnumDiscriminants};
pub use system_message::*;
pub use target_unselected::*;
pub use teleport_to_location::*;
pub use user_info::*;
pub use validate_location::*;

pub struct GameServerPacketCodes;
impl GameServerPacketCodes {
    const DIE: ServerPacketId = ServerPacketId::new(0x00);
    const REVIVE: ServerPacketId = ServerPacketId::new(0x01);
    const _ATTACK_OUTOF_RANGE: ServerPacketId = ServerPacketId::new(0x02);
    const _ATTACKIN_COOL_TIME: ServerPacketId = ServerPacketId::new(0x03);
    const _ATTACK_DEAD_TARGET: ServerPacketId = ServerPacketId::new(0x04);
    const SPAWN_ITEM: ServerPacketId = ServerPacketId::new(0x05);
    const DELETE_OBJECT: ServerPacketId = ServerPacketId::new(0x08);
    const CHARACTER_SELECTION_INFO: ServerPacketId = ServerPacketId::new(0x09);
    const _LOGIN_FAIL: ServerPacketId = ServerPacketId::new(0x0A);
    const CHARACTER_SELECTED: ServerPacketId = ServerPacketId::new(0x0B);
    const NPC_INFO: ServerPacketId = ServerPacketId::new(0x0C);
    const NEW_CHARACTER_SUCCESS: ServerPacketId = ServerPacketId::new(0x0D);
    const _NEW_CHARACTER_FAIL: ServerPacketId = ServerPacketId::new(0x0E);
    const CHARACTER_CREATE_SUCCESS: ServerPacketId = ServerPacketId::new(0x0F);
    const CHARACTER_CREATE_FAIL: ServerPacketId = ServerPacketId::new(0x10);
    const ITEM_LIST: ServerPacketId = ServerPacketId::new(0x11);
    const _SUNRISE: ServerPacketId = ServerPacketId::new(0x12);
    const _SUNSET: ServerPacketId = ServerPacketId::new(0x13);
    const _TRADE_START: ServerPacketId = ServerPacketId::new(0x14);
    const _TRADE_START_OK: ServerPacketId = ServerPacketId::new(0x15);
    const DROP_ITEM: ServerPacketId = ServerPacketId::new(0x16);
    const GET_ITEM: ServerPacketId = ServerPacketId::new(0x17);
    const STATUS_UPDATE: ServerPacketId = ServerPacketId::new(0x18);
    const NPC_HTML_MESSAGE: ServerPacketId = ServerPacketId::new(0x19);
    const _TRADE_OWN_ADD: ServerPacketId = ServerPacketId::new(0x1A);
    const _TRADE_OTHER_ADD: ServerPacketId = ServerPacketId::new(0x1B);
    const _TRADE_DONE: ServerPacketId = ServerPacketId::new(0x1C);
    const CHARACTER_DELETE_SUCCESS: ServerPacketId = ServerPacketId::new(0x1D);
    const CHARACTER_DELETE_FAIL: ServerPacketId = ServerPacketId::new(0x1E);
    const ACTION_FAIL: ServerPacketId = ServerPacketId::new(0x1F);
    const _SERVER_CLOSE: ServerPacketId = ServerPacketId::new(0x20);
    const INVENTORY_UPDATE: ServerPacketId = ServerPacketId::new(0x21);
    const TELEPORT_TO_LOCATION: ServerPacketId = ServerPacketId::new(0x22);
    const _TARGET_SELECTED: ServerPacketId = ServerPacketId::new(0x23);
    const TARGET_UNSELECTED: ServerPacketId = ServerPacketId::new(0x24);
    const ATTACK_STANCE_START: ServerPacketId = ServerPacketId::new(0x25);
    const ATTACK_STANCE_STOP: ServerPacketId = ServerPacketId::new(0x26);
    const SOCIAL_ACTION: ServerPacketId = ServerPacketId::new(0x27);
    const CHANGE_MOVE_TYPE: ServerPacketId = ServerPacketId::new(0x28);
    const CHANGE_WAIT_TYPE: ServerPacketId = ServerPacketId::new(0x29);
    const _MANAGE_PLEDGE_POWER: ServerPacketId = ServerPacketId::new(0x2A);
    const _CREATE_PLEDGE: ServerPacketId = ServerPacketId::new(0x2B);
    const _ASK_JOIN_PLEDGE: ServerPacketId = ServerPacketId::new(0x2C);
    const _JOIN_PLEDGE: ServerPacketId = ServerPacketId::new(0x2D);
    const KEY_PACKET: ServerPacketId = ServerPacketId::new(0x2e);
    const MOVE_TO_LOCATION: ServerPacketId = ServerPacketId::new(0x2F);
    const _NPC_SAY: ServerPacketId = ServerPacketId::new(0x30);
    const CHAR_INFO: ServerPacketId = ServerPacketId::new(0x31);
    const USER_INFO: ServerPacketId = ServerPacketId::new(0x32);
    const ATTACK: ServerPacketId = ServerPacketId::new(0x33);
    const _WITHDRAWAL_PLEDGE: ServerPacketId = ServerPacketId::new(0x34);
    const _OUST_PLEDGE_MEMBER: ServerPacketId = ServerPacketId::new(0x35);
    const _SET_OUST_PLEDGE_MEMBER: ServerPacketId = ServerPacketId::new(0x36);
    const _DISMISS_PLEDGE: ServerPacketId = ServerPacketId::new(0x37);
    const _SET_DISMISS_PLEDGE: ServerPacketId = ServerPacketId::new(0x38);
    const _ASK_JOIN_PARTY: ServerPacketId = ServerPacketId::new(0x39);
    const _JOIN_PARTY: ServerPacketId = ServerPacketId::new(0x3A);
    const _WITHDRAWAL_PARTY: ServerPacketId = ServerPacketId::new(0x3B);
    const _OUST_PARTY_MEMBER: ServerPacketId = ServerPacketId::new(0x3C);
    const _SET_OUST_PARTY_MEMBER: ServerPacketId = ServerPacketId::new(0x3D);
    const _DISMISS_PARTY: ServerPacketId = ServerPacketId::new(0x3E);
    const _SET_DISMISS_PARTY: ServerPacketId = ServerPacketId::new(0x3F);
    const _USER_ACK: ServerPacketId = ServerPacketId::new(0x40);
    const _WAREHOUSE_DEPOSIT_LIST: ServerPacketId = ServerPacketId::new(0x41);
    const _WAREHOUSE_WITHDRAW_LIST: ServerPacketId = ServerPacketId::new(0x42);
    const _WAREHOUSE_DONE: ServerPacketId = ServerPacketId::new(0x43);
    const SHORT_CUT_REGISTER: ServerPacketId = ServerPacketId::new(0x44);
    const SHORT_CUT_INIT: ServerPacketId = ServerPacketId::new(0x45);
    const _SHORT_CUT_DELETE: ServerPacketId = ServerPacketId::new(0x46);
    const STOP_MOVE: ServerPacketId = ServerPacketId::new(0x47);
    const MAGIC_SKILL_USE: ServerPacketId = ServerPacketId::new(0x48);
    const MAGIC_SKILL_CANCELED: ServerPacketId = ServerPacketId::new(0x49);
    const CREATURE_SAY: ServerPacketId = ServerPacketId::new(0x4A);
    const _EQUIP_UPDATE: ServerPacketId = ServerPacketId::new(0x4B);
    const _DOOR_INFO: ServerPacketId = ServerPacketId::new(0x4C);
    const DOOR_STATUS_UPDATE: ServerPacketId = ServerPacketId::new(0x4D);
    const _PARTY_SMALL_WINDOW_ALL: ServerPacketId = ServerPacketId::new(0x4E);
    const _PARTY_SMALL_WINDOW_ADD: ServerPacketId = ServerPacketId::new(0x4F);
    const _PARTY_SMALL_WINDOW_DELETE_ALL: ServerPacketId = ServerPacketId::new(0x50);
    const _PARTY_SMALL_WINDOW_DELETE: ServerPacketId = ServerPacketId::new(0x51);
    const _PARTY_SMALL_WINDOW_UPDATE: ServerPacketId = ServerPacketId::new(0x52);
    const _TRADE_PRESS_OWN_OK: ServerPacketId = ServerPacketId::new(0x53);
    const MAGIC_SKILL_LAUNCHED: ServerPacketId = ServerPacketId::new(0x54);
    const _FRIEND_ADD_REQUEST_RESULT: ServerPacketId = ServerPacketId::new(0x55);
    const _FRIEND_ADD: ServerPacketId = ServerPacketId::new(0x56);
    const _FRIEND_REMOVE: ServerPacketId = ServerPacketId::new(0x57);
    const _FRIEND_LIST: ServerPacketId = ServerPacketId::new(0x58);
    const _FRIEND_STATUS: ServerPacketId = ServerPacketId::new(0x59);
    const _PLEDGE_SHOW_MEMBER_LIST_ALL: ServerPacketId = ServerPacketId::new(0x5A);
    const _PLEDGE_SHOW_MEMBER_LIST_UPDATE: ServerPacketId = ServerPacketId::new(0x5B);
    const _PLEDGE_SHOW_MEMBER_LIST_ADD: ServerPacketId = ServerPacketId::new(0x5C);
    const _PLEDGE_SHOW_MEMBER_LIST_DELETE: ServerPacketId = ServerPacketId::new(0x5D);
    const _MAGIC_LIST: ServerPacketId = ServerPacketId::new(0x5E);
    const SKILL_LIST: ServerPacketId = ServerPacketId::new(0x5F);
    const _VEHICLE_INFO: ServerPacketId = ServerPacketId::new(0x60);
    const _FINISH_ROTATING: ServerPacketId = ServerPacketId::new(0x61);
    const SYSTEM_MESSAGE: ServerPacketId = ServerPacketId::new(0x62);
    const _START_PLEDGE_WAR: ServerPacketId = ServerPacketId::new(0x63);
    const _REPLY_START_PLEDGE_WAR: ServerPacketId = ServerPacketId::new(0x64);
    const _STOP_PLEDGE_WAR: ServerPacketId = ServerPacketId::new(0x65);
    const _REPLY_STOP_PLEDGE_WAR: ServerPacketId = ServerPacketId::new(0x66);
    const _SURRENDER_PLEDGE_WAR: ServerPacketId = ServerPacketId::new(0x67);
    const _REPLY_SURRENDER_PLEDGE_WAR: ServerPacketId = ServerPacketId::new(0x68);
    const _SET_PLEDGE_CREST: ServerPacketId = ServerPacketId::new(0x69);
    const _PLEDGE_CREST: ServerPacketId = ServerPacketId::new(0x6A);
    const SETUP_GAUGE: ServerPacketId = ServerPacketId::new(0x6B);
    const _VEHICLE_DEPARTURE: ServerPacketId = ServerPacketId::new(0x6C);
    const _VEHICLE_CHECK_LOCATION: ServerPacketId = ServerPacketId::new(0x6D);
    const _GET_ON_VEHICLE: ServerPacketId = ServerPacketId::new(0x6E);
    const _GET_OFF_VEHICLE: ServerPacketId = ServerPacketId::new(0x6F);
    const _TRADE_REQUEST: ServerPacketId = ServerPacketId::new(0x70);
    const RESTART_RESPONSE: ServerPacketId = ServerPacketId::new(0x71);
    const MOVE_TO_PAWN: ServerPacketId = ServerPacketId::new(0x72);
    const SSQ_INFO: ServerPacketId = ServerPacketId::new(0x73);
    const _GAME_GUARD_QUERY: ServerPacketId = ServerPacketId::new(0x74);
    const _L2_FRIEND_LIST: ServerPacketId = ServerPacketId::new(0x75);
    const _L2_FRIEND: ServerPacketId = ServerPacketId::new(0x76);
    const _L2_FRIEND_STATUS: ServerPacketId = ServerPacketId::new(0x77);
    const _L2_FRIEND_SAY: ServerPacketId = ServerPacketId::new(0x78);
    const VALIDATE_LOCATION: ServerPacketId = ServerPacketId::new(0x79);
    const _START_ROTATING: ServerPacketId = ServerPacketId::new(0x7A);
    const _SHOW_BOARD: ServerPacketId = ServerPacketId::new(0x7B);
    const _CHOOSE_INVENTORY_ITEM: ServerPacketId = ServerPacketId::new(0x7C);
    const _DUMMY: ServerPacketId = ServerPacketId::new(0x7D);
    const _MOVE_TO_LOCATION_IN_VEHICLE: ServerPacketId = ServerPacketId::new(0x7E);
    const _STOP_MOVE_IN_VEHICLE: ServerPacketId = ServerPacketId::new(0x7F);
    const _VALIDATE_LOCATION_IN_VEHICLE: ServerPacketId = ServerPacketId::new(0x80);
    const _TRADE_UPDATE: ServerPacketId = ServerPacketId::new(0x81);
    const _TRADE_PRESS_OTHER_OK: ServerPacketId = ServerPacketId::new(0x82);
    const _FRIEND_ADD_REQUEST: ServerPacketId = ServerPacketId::new(0x83);
    const LOG_OUT_OK: ServerPacketId = ServerPacketId::new(0x84);
    const ABNORMAL_STATUS_UPDATE: ServerPacketId = ServerPacketId::new(0x85);
    const _QUEST_LIST: ServerPacketId = ServerPacketId::new(0x86);
    const _ENCHANT_RESULT: ServerPacketId = ServerPacketId::new(0x87);
    const _PLEDGE_SHOW_MEMBER_LIST_DELETE_ALL: ServerPacketId = ServerPacketId::new(0x88);
    const _PLEDGE_INFO: ServerPacketId = ServerPacketId::new(0x89);
    const _PLEDGE_EXTENDED_INFO: ServerPacketId = ServerPacketId::new(0x8A);
    const _SURRENDER_PERSONALLY: ServerPacketId = ServerPacketId::new(0x8B);
    const _RIDE: ServerPacketId = ServerPacketId::new(0x8C);
    const _GIVE_NICK_NAME_DONE: ServerPacketId = ServerPacketId::new(0x8D);
    const _PLEDGE_SHOW_INFO_UPDATE: ServerPacketId = ServerPacketId::new(0x8E);
    const _CLIENT_ACTION: ServerPacketId = ServerPacketId::new(0x8F);
    const _ACQUIRE_SKILL_LIST: ServerPacketId = ServerPacketId::new(0x90);
    const _ACQUIRE_SKILL_INFO: ServerPacketId = ServerPacketId::new(0x91);
    const _SERVER_OBJECT_INFO: ServerPacketId = ServerPacketId::new(0x92);
    const _GM_HIDE: ServerPacketId = ServerPacketId::new(0x93);
    const _ACQUIRE_SKILL_DONE: ServerPacketId = ServerPacketId::new(0x94);
    const _GM_VIEW_CHARACTER_INFO: ServerPacketId = ServerPacketId::new(0x95);
    const _GM_VIEW_PLEDGE_INFO: ServerPacketId = ServerPacketId::new(0x96);
    const _GM_VIEW_SKILL_INFO: ServerPacketId = ServerPacketId::new(0x97);
    const _GM_VIEW_MAGIC_INFO: ServerPacketId = ServerPacketId::new(0x98);
    const _GM_VIEW_QUEST_INFO: ServerPacketId = ServerPacketId::new(0x99);
    const _GM_VIEW_ITEM_LIST: ServerPacketId = ServerPacketId::new(0x9A);
    const _GM_VIEW_WAREHOUSE_WITHDRAW_LIST: ServerPacketId = ServerPacketId::new(0x9B);
    const _LIST_PARTY_WAITING: ServerPacketId = ServerPacketId::new(0x9C);
    const _PARTY_ROOM_INFO: ServerPacketId = ServerPacketId::new(0x9D);
    const PLAY_SOUND: ServerPacketId = ServerPacketId::new(0x9E);
    const STATIC_OBJECT: ServerPacketId = ServerPacketId::new(0x9F);
    const _PRIVATE_STORE_SELL_MANAGE_LIST: ServerPacketId = ServerPacketId::new(0xA0);
    const _PRIVATE_STORE_SELL_LIST: ServerPacketId = ServerPacketId::new(0xA1);
    const _PRIVATE_STORE_SELL_MSG: ServerPacketId = ServerPacketId::new(0xA2);
    const SHOW_MAP: ServerPacketId = ServerPacketId::new(0xA3);
    const _REVIVE_REQUEST: ServerPacketId = ServerPacketId::new(0xA4);
    const _ABNORMAL_VISUAL_EFFECT: ServerPacketId = ServerPacketId::new(0xA5);
    const _TUTORIAL_SHOW_HTML: ServerPacketId = ServerPacketId::new(0xA6);
    const _SHOW_TUTORIAL_MARK: ServerPacketId = ServerPacketId::new(0xA7);
    const _TUTORIAL_ENABLE_CLIENT_EVENT: ServerPacketId = ServerPacketId::new(0xA8);
    const _TUTORIAL_CLOSE_HTML: ServerPacketId = ServerPacketId::new(0xA9);
    const _SHOW_RADAR: ServerPacketId = ServerPacketId::new(0xAA);
    const _WITHDRAW_ALLIANCE: ServerPacketId = ServerPacketId::new(0xAB);
    const _OUST_ALLIANCE_MEMBER_PLEDGE: ServerPacketId = ServerPacketId::new(0xAC);
    const _DISMISS_ALLIANCE: ServerPacketId = ServerPacketId::new(0xAD);
    const _SET_ALLIANCE_CREST: ServerPacketId = ServerPacketId::new(0xAE);
    const _ALLIANCE_CREST: ServerPacketId = ServerPacketId::new(0xAF);
    const _SERVER_CLOSE_SOCKET: ServerPacketId = ServerPacketId::new(0xB0);
    const _PET_STATUS_SHOW: ServerPacketId = ServerPacketId::new(0xB1);
    const _PET_INFO: ServerPacketId = ServerPacketId::new(0xB2);
    const _PET_ITEM_LIST: ServerPacketId = ServerPacketId::new(0xB3);
    const _PET_INVENTORY_UPDATE: ServerPacketId = ServerPacketId::new(0xB4);
    const _ALLIANCE_INFO: ServerPacketId = ServerPacketId::new(0xB5);
    const _PET_STATUS_UPDATE: ServerPacketId = ServerPacketId::new(0xB6);
    const _PET_DELETE: ServerPacketId = ServerPacketId::new(0xB7);
    const _DELETE_RADAR: ServerPacketId = ServerPacketId::new(0xB8);
    const SELECT_TARGET: ServerPacketId = ServerPacketId::new(0xB9);
    const _PARTY_MEMBER_POSITION: ServerPacketId = ServerPacketId::new(0xBA);
    const _ASK_JOIN_ALLIANCE: ServerPacketId = ServerPacketId::new(0xBB);
    const _JOIN_ALLIANCE: ServerPacketId = ServerPacketId::new(0xBC);
    const _PRIVATE_STORE_BUY_MANAGE_LIST: ServerPacketId = ServerPacketId::new(0xBD);
    const _PRIVATE_STORE_BUY_LIST: ServerPacketId = ServerPacketId::new(0xBE);
    const _PRIVATE_STORE_BUY_MSG: ServerPacketId = ServerPacketId::new(0xBF);
    const _VEHICLE_START: ServerPacketId = ServerPacketId::new(0xC0);
    const _REQUEST_TIME_CHECK: ServerPacketId = ServerPacketId::new(0xC1);
    const _START_ALLIANCE_WAR: ServerPacketId = ServerPacketId::new(0xC2);
    const _REPLY_START_ALLIANCE_WAR: ServerPacketId = ServerPacketId::new(0xC3);
    const _STOP_ALLIANCE_WAR: ServerPacketId = ServerPacketId::new(0xC4);
    const _REPLY_STOP_ALLIANCE_WAR: ServerPacketId = ServerPacketId::new(0xC5);
    const _SURRENDER_ALLIANCE_WAR: ServerPacketId = ServerPacketId::new(0xC6);
    const _SKILL_COOL_TIME: ServerPacketId = ServerPacketId::new(0xC7);
    const _PACKAGE_TO_LIST: ServerPacketId = ServerPacketId::new(0xC8);
    const _CASTLE_SIEGE_INFO: ServerPacketId = ServerPacketId::new(0xC9);
    const _CASTLE_SIEGE_ATTACKER_LIST: ServerPacketId = ServerPacketId::new(0xCA);
    const _CASTLE_SIEGE_DEFENDER_LIST: ServerPacketId = ServerPacketId::new(0xCB);
    const _NICK_NAME_CHANGED: ServerPacketId = ServerPacketId::new(0xCC);
    const _PLEDGE_STATUS_CHANGED: ServerPacketId = ServerPacketId::new(0xCD);
    const _RELATION_CHANGED: ServerPacketId = ServerPacketId::new(0xCE);
    const _EVENT_TRIGGER: ServerPacketId = ServerPacketId::new(0xCF);
    const MULTI_SELL_LIST: ServerPacketId = ServerPacketId::new(0xD0);
    const _SET_SUMMON_REMAIN_TIME: ServerPacketId = ServerPacketId::new(0xD1);
    const _PACKAGE_SENDABLE_LIST: ServerPacketId = ServerPacketId::new(0xD2);
    const _EARTHQUAKE: ServerPacketId = ServerPacketId::new(0xD3);
    const _FLY_TO_LOCATION: ServerPacketId = ServerPacketId::new(0xD4);
    const _BLOCK_LIST: ServerPacketId = ServerPacketId::new(0xD5);
    const _SPECIAL_CAMERA: ServerPacketId = ServerPacketId::new(0xD6);
    const _NORMAL_CAMERA: ServerPacketId = ServerPacketId::new(0xD7);
    const _SKILL_REMAIN_SEC: ServerPacketId = ServerPacketId::new(0xD8);
    const NET_PING: ServerPacketId = ServerPacketId::new(0xD9);
    const _DICE: ServerPacketId = ServerPacketId::new(0xDA);
    const _SNOOP: ServerPacketId = ServerPacketId::new(0xDB);
    const _RECIPE_BOOK_ITEM_LIST: ServerPacketId = ServerPacketId::new(0xDC);
    const _RECIPE_ITEM_MAKE_INFO: ServerPacketId = ServerPacketId::new(0xDD);
    const _RECIPE_SHOP_MANAGE_LIST: ServerPacketId = ServerPacketId::new(0xDE);
    const _RECIPE_SHOP_SELL_LIST: ServerPacketId = ServerPacketId::new(0xDF);
    const _RECIPE_SHOP_ITEM_INFO: ServerPacketId = ServerPacketId::new(0xE0);
    const _RECIPE_SHOP_MSG: ServerPacketId = ServerPacketId::new(0xE1);
    const _SHOW_CALC: ServerPacketId = ServerPacketId::new(0xE2);
    const _MON_RACE_INFO: ServerPacketId = ServerPacketId::new(0xE3);
    const _HENNA_ITEM_INFO: ServerPacketId = ServerPacketId::new(0xE4);
    const _HENNA_INFO: ServerPacketId = ServerPacketId::new(0xE5);
    const _HENNA_UNEQUIP_LIST: ServerPacketId = ServerPacketId::new(0xE6);
    const _HENNA_UNEQUIP_INFO: ServerPacketId = ServerPacketId::new(0xE7);
    const _MACRO_LIST: ServerPacketId = ServerPacketId::new(0xE8);
    const _BUY_LIST_SEED: ServerPacketId = ServerPacketId::new(0xE9);
    const _SHOW_TOWN_MAP: ServerPacketId = ServerPacketId::new(0xEA);
    const _OBSERVER_START: ServerPacketId = ServerPacketId::new(0xEB);
    const _OBSERVER_END: ServerPacketId = ServerPacketId::new(0xEC);
    const _CHAIR_SIT: ServerPacketId = ServerPacketId::new(0xED);
    const _HENNA_EQUIP_LIST: ServerPacketId = ServerPacketId::new(0xEE);
    const _SELL_LIST_PROCURE: ServerPacketId = ServerPacketId::new(0xEF);
    const _GM_HENNA_INFO: ServerPacketId = ServerPacketId::new(0xF0);
    const _RADAR_CONTROL: ServerPacketId = ServerPacketId::new(0xF1);
    const _CLIENT_SET_TIME: ServerPacketId = ServerPacketId::new(0xF2);
    const _CONFIRM_DLG: ServerPacketId = ServerPacketId::new(0xF3);
    const _PARTY_SPELLED: ServerPacketId = ServerPacketId::new(0xF4);
    const _SHOP_PREVIEW_LIST: ServerPacketId = ServerPacketId::new(0xF5);
    const _SHOP_PREVIEW_INFO: ServerPacketId = ServerPacketId::new(0xF6);
    const _CAMERA_MODE: ServerPacketId = ServerPacketId::new(0xF7);
    const _SHOW_XMAS_SEAL: ServerPacketId = ServerPacketId::new(0xF8);
    const ETC_STATUS_UPDATE: ServerPacketId = ServerPacketId::new(0xF9);
    const _SHORT_BUFF_STATUS_UPDATE: ServerPacketId = ServerPacketId::new(0xFA);
    const _SSQ_STATUS: ServerPacketId = ServerPacketId::new(0xFB);
    const _PETITION_VOTE: ServerPacketId = ServerPacketId::new(0xFC);
    const _AGIT_DECO_INFO: ServerPacketId = ServerPacketId::new(0xFD);
    // ex packets
    const _EX_DUMMY: ServerPacketId = ServerPacketId::new_ex(0x00);
    const _EX_REGEN_MAX: ServerPacketId = ServerPacketId::new_ex(0x01);
    const _EX_EVENT_MATCH_USER_INFO: ServerPacketId = ServerPacketId::new_ex(0x02);
    const _EX_COLOSSEUM_FENCE_INFO: ServerPacketId = ServerPacketId::new_ex(0x03);
    const _EX_EVENT_MATCH_SPELLED_INFO: ServerPacketId = ServerPacketId::new_ex(0x04);
    const _EX_EVENT_MATCH_FIRECRACKER: ServerPacketId = ServerPacketId::new_ex(0x05);
    const _EX_EVENT_MATCH_TEAM_UNLOCKED: ServerPacketId = ServerPacketId::new_ex(0x06);
    const _EX_EVENT_MATCH_GM_TEST: ServerPacketId = ServerPacketId::new_ex(0x07);
    const _EX_PARTY_ROOM_MEMBER: ServerPacketId = ServerPacketId::new_ex(0x08);
    const _EX_CLOSE_PARTY_ROOM: ServerPacketId = ServerPacketId::new_ex(0x09);
    const _EX_MANAGE_PARTY_ROOM_MEMBER: ServerPacketId = ServerPacketId::new_ex(0x0A);
    const _EX_EVENT_MATCH_LOCK_RESULT: ServerPacketId = ServerPacketId::new_ex(0x0B);
    const EX_AUTO_SHOTS: ServerPacketId = ServerPacketId::new_ex(0x0C);
    const _EX_EVENT_MATCH_LIST: ServerPacketId = ServerPacketId::new_ex(0x0D);
    const _EX_EVENT_MATCH_OBSERVER: ServerPacketId = ServerPacketId::new_ex(0x0E);
    const _EX_EVENT_MATCH_MESSAGE: ServerPacketId = ServerPacketId::new_ex(0x0F);
    const _EX_EVENT_MATCH_SCORE: ServerPacketId = ServerPacketId::new_ex(0x10);
    const _EX_SERVER_PRIMITIVE: ServerPacketId = ServerPacketId::new_ex(0x11);
    const _EX_OPEN_MPCC: ServerPacketId = ServerPacketId::new_ex(0x12);
    const _EX_CLOSE_MPCC: ServerPacketId = ServerPacketId::new_ex(0x13);
    const _EX_SHOW_CASTLE_INFO: ServerPacketId = ServerPacketId::new_ex(0x14);
    const _EX_SHOW_FORTRESS_INFO: ServerPacketId = ServerPacketId::new_ex(0x15);
    const _EX_SHOW_AGIT_INFO: ServerPacketId = ServerPacketId::new_ex(0x16);
    const _EX_SHOW_FORTRESS_SIEGE_INFO: ServerPacketId = ServerPacketId::new_ex(0x17);
    const _EX_PARTY_PET_WINDOW_ADD: ServerPacketId = ServerPacketId::new_ex(0x18);
    const _EX_PARTY_PET_WINDOW_UPDATE: ServerPacketId = ServerPacketId::new_ex(0x19);
    const _EX_ASK_JOIN_MPCC: ServerPacketId = ServerPacketId::new_ex(0x1A);
    const _EX_PLEDGE_EMBLEM: ServerPacketId = ServerPacketId::new_ex(0x1B);
    const _EX_EVENT_MATCH_TEAM_INFO: ServerPacketId = ServerPacketId::new_ex(0x1C);
    const _EX_EVENT_MATCH_CREATE: ServerPacketId = ServerPacketId::new_ex(0x1D);
    const _EX_FISHING_START: ServerPacketId = ServerPacketId::new_ex(0x1E);
    const _EX_FISHING_END: ServerPacketId = ServerPacketId::new_ex(0x1F);
    const _EX_SHOW_QUEST_INFO: ServerPacketId = ServerPacketId::new_ex(0x20);
    const _EX_SHOW_QUEST_MARK: ServerPacketId = ServerPacketId::new_ex(0x21);
    const _EX_SEND_MANOR_LIST: ServerPacketId = ServerPacketId::new_ex(0x22);
    const _EX_SHOW_SEED_INFO: ServerPacketId = ServerPacketId::new_ex(0x23);
    const _EX_SHOW_CROP_INFO: ServerPacketId = ServerPacketId::new_ex(0x24);
    const _EX_SHOW_MANOR_DEFAULT_INFO: ServerPacketId = ServerPacketId::new_ex(0x25);
    const _EX_SHOW_SEED_SETTING: ServerPacketId = ServerPacketId::new_ex(0x26);
    const _EX_FISHING_START_COMBAT: ServerPacketId = ServerPacketId::new_ex(0x27);
    const _EX_FISHING_HP_REGEN: ServerPacketId = ServerPacketId::new_ex(0x28);
    const _EX_ENCHANT_SKILL_LIST: ServerPacketId = ServerPacketId::new_ex(0x29);
    const _EX_ENCHANT_SKILL_INFO: ServerPacketId = ServerPacketId::new_ex(0x2A);
    const _EX_SHOW_CROP_SETTING: ServerPacketId = ServerPacketId::new_ex(0x2B);
    const _EX_SHOW_SELL_CROP_LIST: ServerPacketId = ServerPacketId::new_ex(0x2C);
    const _EX_OLYMPIAD_MATCH_END: ServerPacketId = ServerPacketId::new_ex(0x2D);
    const _EX_MAIL_ARRIVED: ServerPacketId = ServerPacketId::new_ex(0x2E);
    const _EX_STORAGE_MAX_COUNT: ServerPacketId = ServerPacketId::new_ex(0x2F);
    const _EX_EVENT_MATCH_MANAGE: ServerPacketId = ServerPacketId::new_ex(0x30);
    const _EX_MULTI_PARTY_COMMAND_CHANNEL_INFO: ServerPacketId = ServerPacketId::new_ex(0x31);
    const _EX_PC_CAFE_POINT_INFO: ServerPacketId = ServerPacketId::new_ex(0x32);
    const _EX_SET_COMPASS_ZONE_CODE: ServerPacketId = ServerPacketId::new_ex(0x33);
    const _EX_GET_BOSS_RECORD: ServerPacketId = ServerPacketId::new_ex(0x34);
    const _EX_ASK_JOIN_PARTY_ROOM: ServerPacketId = ServerPacketId::new_ex(0x35);
    const _EX_LIST_PARTY_MATCHING_WAITING_ROOM: ServerPacketId = ServerPacketId::new_ex(0x36);
    const _EX_SET_MPCC_ROUTING: ServerPacketId = ServerPacketId::new_ex(0x37);
    const _EX_SHOW_ADVENTURER_GUIDE_BOOK: ServerPacketId = ServerPacketId::new_ex(0x38);
    const _EX_SHOW_SCREEN_MESSAGE: ServerPacketId = ServerPacketId::new_ex(0x39);
    const _PLEDGE_SKILL_LIST: ServerPacketId = ServerPacketId::new_ex(0x3A);
    const _PLEDGE_SKILL_LIST_ADD: ServerPacketId = ServerPacketId::new_ex(0x3B);
    const _PLEDGE_POWER_GRADE_LIST: ServerPacketId = ServerPacketId::new_ex(0x3C);
    const _PLEDGE_RECEIVE_POWER_INFO: ServerPacketId = ServerPacketId::new_ex(0x3D);
    const _PLEDGE_RECEIVE_MEMBER_INFO: ServerPacketId = ServerPacketId::new_ex(0x3E);
    const _PLEDGE_RECEIVE_WAR_LIST: ServerPacketId = ServerPacketId::new_ex(0x3F);
    const _PLEDGE_RECEIVE_SUB_PLEDGE_CREATED: ServerPacketId = ServerPacketId::new_ex(0x40);
    const _EX_RED_SKY: ServerPacketId = ServerPacketId::new_ex(0x41);
    const _PLEDGE_RECEIVE_UPDATE_POWER: ServerPacketId = ServerPacketId::new_ex(0x42);
    const _FLY_SELF_DESTINATION: ServerPacketId = ServerPacketId::new_ex(0x43);
    const _SHOW_PC_CAFE_COUPON_SHOW_UI: ServerPacketId = ServerPacketId::new_ex(0x44);
    const _EX_SEARCH_ORC: ServerPacketId = ServerPacketId::new_ex(0x45);
    const _EX_CURSED_WEAPON_LIST: ServerPacketId = ServerPacketId::new_ex(0x46);
    const _EX_CURSED_WEAPON_LOCATION: ServerPacketId = ServerPacketId::new_ex(0x47);
    const _EX_RESTART_CLIENT: ServerPacketId = ServerPacketId::new_ex(0x48);
    const _EX_REQUEST_HACK_SHIELD: ServerPacketId = ServerPacketId::new_ex(0x49);
    const _EX_USE_SHARED_GROUP_ITEM: ServerPacketId = ServerPacketId::new_ex(0x4A);
    const _EX_MPCC_SHOW_PARTY_MEMBER_INFO: ServerPacketId = ServerPacketId::new_ex(0x4B);
    const _EX_DUEL_ASK_START: ServerPacketId = ServerPacketId::new_ex(0x4C);
    const _EX_DUEL_READY: ServerPacketId = ServerPacketId::new_ex(0x4D);
    const _EX_DUEL_START: ServerPacketId = ServerPacketId::new_ex(0x4E);
    const _EX_DUEL_END: ServerPacketId = ServerPacketId::new_ex(0x4F);
    const _EX_DUEL_UPDATE_USER_INFO: ServerPacketId = ServerPacketId::new_ex(0x50);
    const _EX_SHOW_VARIATION_MAKE_WINDOW: ServerPacketId = ServerPacketId::new_ex(0x51);
    const _EX_SHOW_VARIATION_CANCEL_WINDOW: ServerPacketId = ServerPacketId::new_ex(0x52);
    const _EX_PUT_ITEM_RESULT_FOR_VARIATION_MAKE: ServerPacketId = ServerPacketId::new_ex(0x53);
    const _EX_PUT_INTENSIVE_RESULT_FOR_VARIATION_MAKE: ServerPacketId =
        ServerPacketId::new_ex(0x54);
    const _EX_PUT_COMMISSION_RESULT_FOR_VARIATION_MAKE: ServerPacketId =
        ServerPacketId::new_ex(0x55);
    const _EX_VARIATION_RESULT: ServerPacketId = ServerPacketId::new_ex(0x56);
    const _EX_PUT_ITEM_RESULT_FOR_VARIATION_CANCEL: ServerPacketId = ServerPacketId::new_ex(0x57);
    const _EX_VARIATION_CANCEL_RESULT: ServerPacketId = ServerPacketId::new_ex(0x58);
    const _EX_DUEL_ENEMY_RELATION: ServerPacketId = ServerPacketId::new_ex(0x59);
    const _EX_PLAY_ANIMATION: ServerPacketId = ServerPacketId::new_ex(0x5A);
    const _EX_MPCC_PARTY_INFO_UPDATE: ServerPacketId = ServerPacketId::new_ex(0x5B);
    const _EX_PLAY_SCENE: ServerPacketId = ServerPacketId::new_ex(0x5C);
    const _EX_SPAWN_EMITTER: ServerPacketId = ServerPacketId::new_ex(0x5D);
    const _EX_ENCHANT_SKILL_INFO_DETAIL: ServerPacketId = ServerPacketId::new_ex(0x5E);
    const EX_BASIC_ACTION_LIST: ServerPacketId = ServerPacketId::new_ex(0x5F);
    const _EX_AIRSHIP_INFO: ServerPacketId = ServerPacketId::new_ex(0x60);
    const _EX_ATTRIBUTE_ENCHANT_RESULT: ServerPacketId = ServerPacketId::new_ex(0x61);
    const _EX_CHOOSE_INVENTORY_ATTRIBUTE_ITEM: ServerPacketId = ServerPacketId::new_ex(0x62);
    const _EX_GET_ON_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x63);
    const _EX_GET_OFF_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x64);
    const _EX_MOVE_TO_LOCATION_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x65);
    const _EX_STOP_MOVE_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x66);
    const _EX_SHOW_TRACE: ServerPacketId = ServerPacketId::new_ex(0x67);
    const _EX_ITEM_AUCTION_INFO: ServerPacketId = ServerPacketId::new_ex(0x68);
    const _EX_NEED_TO_CHANGE_NAME: ServerPacketId = ServerPacketId::new_ex(0x69);
    const _EX_PARTY_PET_WINDOW_DELETE: ServerPacketId = ServerPacketId::new_ex(0x6A);
    const _EX_TUTORIAL_LIST: ServerPacketId = ServerPacketId::new_ex(0x6B);
    const _EX_RP_ITEM_LINK: ServerPacketId = ServerPacketId::new_ex(0x6C);
    const _EX_MOVE_TO_LOCATION_IN_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x6D);
    const _EX_STOP_MOVE_IN_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x6E);
    const _EX_VALIDATE_LOCATION_IN_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x6F);
    const _EX_UI_SETTING: ServerPacketId = ServerPacketId::new_ex(0x70);
    const _EX_MOVE_TO_TARGET_IN_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x71);
    const _EX_ATTACK_IN_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x72);
    const _EX_MAGIC_SKILL_USE_IN_AIRSHIP: ServerPacketId = ServerPacketId::new_ex(0x73);
    const _EX_SHOW_BASE_ATTRIBUTE_CANCEL_WINDOW: ServerPacketId = ServerPacketId::new_ex(0x74);
    const _EX_BASE_ATTRIBUTE_CANCEL_RESULT: ServerPacketId = ServerPacketId::new_ex(0x75);
    const _EX_SUB_PLEDGE_SKILL_ADD: ServerPacketId = ServerPacketId::new_ex(0x76);
    const _EX_RESPONSE_FREE_SERVER: ServerPacketId = ServerPacketId::new_ex(0x77);
    const _EX_SHOW_PROCURE_CROP_DETAIL: ServerPacketId = ServerPacketId::new_ex(0x78);
    const _EX_HERO_LIST: ServerPacketId = ServerPacketId::new_ex(0x79);
    const _EX_OLYMPIAD_USER_INFO: ServerPacketId = ServerPacketId::new_ex(0x7A);
    const _EX_OLYMPIAD_SPELLED_INFO: ServerPacketId = ServerPacketId::new_ex(0x7B);
    const _EX_OLYMPIAD_MODE: ServerPacketId = ServerPacketId::new_ex(0x7C);
    const _EX_SHOW_FORTRESS_MAP_INFO: ServerPacketId = ServerPacketId::new_ex(0x7D);
    const _EX_PVP_MATCH_RECORD: ServerPacketId = ServerPacketId::new_ex(0x7E);
    const _EX_PVP_MATCH_USER_DIE: ServerPacketId = ServerPacketId::new_ex(0x7F);
    const _EX_PRIVATE_STORE_PACKAGE_MSG: ServerPacketId = ServerPacketId::new_ex(0x80);
    const _EX_PUT_ENCHANT_TARGET_ITEM_RESULT: ServerPacketId = ServerPacketId::new_ex(0x81);
    const _EX_PUT_ENCHANT_SUPPORT_ITEM_RESULT: ServerPacketId = ServerPacketId::new_ex(0x82);
    const _EX_REQUEST_CHANGE_NICKNAME_COLOR: ServerPacketId = ServerPacketId::new_ex(0x83);
    const _EX_GET_BOOKMARK_INFO: ServerPacketId = ServerPacketId::new_ex(0x84);
    const _EX_NOTIFY_PREMIUM_ITEM: ServerPacketId = ServerPacketId::new_ex(0x85);
    const _EX_GET_PREMIUM_ITEM_LIST: ServerPacketId = ServerPacketId::new_ex(0x86);
    const _EX_PERIODIC_ITEM_LIST: ServerPacketId = ServerPacketId::new_ex(0x87);
    const _EX_JUMP_TO_LOCATION: ServerPacketId = ServerPacketId::new_ex(0x88);
    const _EX_PVP_MATCH_CC_RECORD: ServerPacketId = ServerPacketId::new_ex(0x89);
    const _EX_PVP_MATCH_CC_MY_RECORD: ServerPacketId = ServerPacketId::new_ex(0x8A);
    const _EX_PVP_MATCH_CC_RETIRE: ServerPacketId = ServerPacketId::new_ex(0x8B);
    const _EX_SHOW_TERRITORY: ServerPacketId = ServerPacketId::new_ex(0x8C);
    const _EX_NPC_QUEST_HTML_MESSAGE: ServerPacketId = ServerPacketId::new_ex(0x8D);
    const _EX_SEND_UI_EVENT: ServerPacketId = ServerPacketId::new_ex(0x8E);
    const _EX_NOTIFY_BIRTHDAY: ServerPacketId = ServerPacketId::new_ex(0x8F);
    const _EX_SHOW_DOMINION_REGISTRY: ServerPacketId = ServerPacketId::new_ex(0x90);
    const _EX_REPLY_REGISTER_DOMINION: ServerPacketId = ServerPacketId::new_ex(0x91);
    const _EX_REPLY_DOMINION_INFO: ServerPacketId = ServerPacketId::new_ex(0x92);
    const _EX_SHOW_OWNTHING_POS: ServerPacketId = ServerPacketId::new_ex(0x93);
    const _EX_CLEFT_LIST: ServerPacketId = ServerPacketId::new_ex(0x94);
    const _EX_CLEFT_STATE: ServerPacketId = ServerPacketId::new_ex(0x95);
    const _EX_DOMINION_CHANNEL_SET: ServerPacketId = ServerPacketId::new_ex(0x96);
    const _EX_BLOCK_UP_SET_LIST: ServerPacketId = ServerPacketId::new_ex(0x97);
    const _EX_BLOCK_UP_SET_STATE: ServerPacketId = ServerPacketId::new_ex(0x98);
    const _EX_START_SCENE_PLAYER: ServerPacketId = ServerPacketId::new_ex(0x99);
    const _EX_AIRSHIP_TELEPORT_LIST: ServerPacketId = ServerPacketId::new_ex(0x9A);
    const _EX_MPCC_ROOM_INFO: ServerPacketId = ServerPacketId::new_ex(0x9B);
    const _EX_LIST_MPCC_WAITING: ServerPacketId = ServerPacketId::new_ex(0x9C);
    const _EX_DISSMISS_MPCC_ROOM: ServerPacketId = ServerPacketId::new_ex(0x9D);
    const _EX_MANAGE_MPCC_ROOM_MEMBER: ServerPacketId = ServerPacketId::new_ex(0x9E);
    const _EX_MPCC_ROOM_MEMBER: ServerPacketId = ServerPacketId::new_ex(0x9F);
    const _EX_VITALITY_POINT_INFO: ServerPacketId = ServerPacketId::new_ex(0xA0);
    const _EX_SHOW_SEED_MAP_INFO: ServerPacketId = ServerPacketId::new_ex(0xA1);
    const _EX_MPCC_PARTYMASTER_LIST: ServerPacketId = ServerPacketId::new_ex(0xA2);
    const _EX_DOMINION_WAR_START: ServerPacketId = ServerPacketId::new_ex(0xA3);
    const _EX_DOMINION_WAR_END: ServerPacketId = ServerPacketId::new_ex(0xA4);
    const _EX_SHOW_LINES: ServerPacketId = ServerPacketId::new_ex(0xA5);
    const _EX_PARTY_MEMBER_RENAMED: ServerPacketId = ServerPacketId::new_ex(0xA6);
    const _EX_ENCHANT_SKILL_RESULT: ServerPacketId = ServerPacketId::new_ex(0xA7);
    const _EX_REFUND_LIST: ServerPacketId = ServerPacketId::new_ex(0xA8);
    const _EX_NOTICE_POST_ARRIVED: ServerPacketId = ServerPacketId::new_ex(0xA9);
    const _EX_SHOW_RECEIVED_POST_LIST: ServerPacketId = ServerPacketId::new_ex(0xAA);
    const _EX_REPLY_RECEIVED_POST: ServerPacketId = ServerPacketId::new_ex(0xAB);
    const _EX_SHOW_SENT_POST_LIST: ServerPacketId = ServerPacketId::new_ex(0xAC);
    const _EX_REPLY_SENT_POST: ServerPacketId = ServerPacketId::new_ex(0xAD);
    const _EX_RESPONSE_SHOW_STEP_ONE: ServerPacketId = ServerPacketId::new_ex(0xAE);
    const _EX_RESPONSE_SHOW_STEP_TWO: ServerPacketId = ServerPacketId::new_ex(0xAF);
    const _EX_RESPONSE_SHOW_CONTENTS: ServerPacketId = ServerPacketId::new_ex(0xB0);
    const _EX_SHOW_PETITION_HTML: ServerPacketId = ServerPacketId::new_ex(0xB1);
    const _EX_REPLY_POST_ITEM_LIST: ServerPacketId = ServerPacketId::new_ex(0xB2);
    const _EX_CHANGE_POST_STATE: ServerPacketId = ServerPacketId::new_ex(0xB3);
    const _EX_NOTICE_POST_SENT: ServerPacketId = ServerPacketId::new_ex(0xB4);
    const _EX_INITIALIZE_SEED: ServerPacketId = ServerPacketId::new_ex(0xB5);
    const _EX_RAID_RESERVE_RESULT: ServerPacketId = ServerPacketId::new_ex(0xB6);
    const _EX_BUY_SELL_LIST: ServerPacketId = ServerPacketId::new_ex(0xB7);
    const _EX_CLOSE_RAID_SOCKET: ServerPacketId = ServerPacketId::new_ex(0xB8);
    const _EX_PRIVATE_MARKET_LIST: ServerPacketId = ServerPacketId::new_ex(0xB9);
    const _EX_RAID_CHARACTER_SELECTED: ServerPacketId = ServerPacketId::new_ex(0xBA);
    const _EX_ASK_COUPLE_ACTION: ServerPacketId = ServerPacketId::new_ex(0xBB);
    const _EX_BR_BROADCAST_EVENT_STATE: ServerPacketId = ServerPacketId::new_ex(0xBC);
    const _EX_BR_LOAD_EVENT_TOP_RANKERS: ServerPacketId = ServerPacketId::new_ex(0xBD);
    const _EX_CHANGE_NPC_STATE: ServerPacketId = ServerPacketId::new_ex(0xBE);
    const _EX_ASK_MODIFY_PARTY_LOOTING: ServerPacketId = ServerPacketId::new_ex(0xBF);
    const _EX_SET_PARTY_LOOTING: ServerPacketId = ServerPacketId::new_ex(0xC0);
    const EX_ROTATION: ServerPacketId = ServerPacketId::new_ex(0xC1);
    const _EX_CHANGE_CLIENT_EFFECT_INFO: ServerPacketId = ServerPacketId::new_ex(0xC2);
    const _EX_MEMBERSHIP_INFO: ServerPacketId = ServerPacketId::new_ex(0xC3);
    const _EX_REPLY_HAND_OVER_PARTY_MASTER: ServerPacketId = ServerPacketId::new_ex(0xC4);
    const _EX_QUEST_NPC_LOG_LIST: ServerPacketId = ServerPacketId::new_ex(0xC5);
    const _EX_QUEST_ITEM_LIST: ServerPacketId = ServerPacketId::new_ex(0xC6);
    const _EX_GM_VIEW_QUEST_ITEM_LIST: ServerPacketId = ServerPacketId::new_ex(0xC7);
    const _EX_RESTART_RESPONSE: ServerPacketId = ServerPacketId::new_ex(0xC8);
    const _EX_VOTE_SYSTEM_INFO: ServerPacketId = ServerPacketId::new_ex(0xC9);
    const _EX_SHUTTLE_INFO: ServerPacketId = ServerPacketId::new_ex(0xCA);
    const _EX_SUTTLE_GET_ON: ServerPacketId = ServerPacketId::new_ex(0xCB);
    const _EX_SUTTLE_GET_OFF: ServerPacketId = ServerPacketId::new_ex(0xCC);
    const _EX_SUTTLE_MOVE: ServerPacketId = ServerPacketId::new_ex(0xCD);
    const _EX_MOVE_TO_LOCATION_IN_SUTTLE: ServerPacketId = ServerPacketId::new_ex(0xCE);
    const _EX_STOP_MOVE_IN_SHUTTLE: ServerPacketId = ServerPacketId::new_ex(0xCF);
    const _EX_VALIDATE_LOCATION_IN_SHUTTLE: ServerPacketId = ServerPacketId::new_ex(0xD0);
    const _EX_AGIT_AUCTION_CMD: ServerPacketId = ServerPacketId::new_ex(0xD1);
    const _EX_CONFIRM_ADDING_POST_FRIEND: ServerPacketId = ServerPacketId::new_ex(0xD2);
    const _EX_RECEIVE_SHOW_POST_FRIEND: ServerPacketId = ServerPacketId::new_ex(0xD3);
    const _EX_RECEIVE_OLYMPIAD: ServerPacketId = ServerPacketId::new_ex(0xD4);
    const _EX_BR_GAME_POINT: ServerPacketId = ServerPacketId::new_ex(0xD5);
    const _EX_BR_PRODUCT_LIST: ServerPacketId = ServerPacketId::new_ex(0xD6);
    const _EX_BR_PRODUCT_INFO: ServerPacketId = ServerPacketId::new_ex(0xD7);
    const _EX_BR_BUY_PRODUCT: ServerPacketId = ServerPacketId::new_ex(0xD8);
    const _EX_BR_PREMIUM_STATE: ServerPacketId = ServerPacketId::new_ex(0xD9);
    const EX_BR_EXTRA_USER_INFO: ServerPacketId = ServerPacketId::new_ex(0xDA);
    const _EX_BR_BUFF_EVENT_STATE: ServerPacketId = ServerPacketId::new_ex(0xDB);
    const _EX_BR_RECENT_PRODUCT_LIST: ServerPacketId = ServerPacketId::new_ex(0xDC);
    const _EX_BR_MINIGAME_LOAD_SCORES: ServerPacketId = ServerPacketId::new_ex(0xDD);
    const _EX_BR_AGATHION_ENERGY_INFO: ServerPacketId = ServerPacketId::new_ex(0xDE);
    const _EX_NAVIT_ADVENT_POINT_INFO: ServerPacketId = ServerPacketId::new_ex(0xDF);
    const _EX_NAVIT_ADVENT_EFFECT: ServerPacketId = ServerPacketId::new_ex(0xE0);
    const _EX_NAVIT_ADVENT_TIME_CHANGE: ServerPacketId = ServerPacketId::new_ex(0xE1);
    const _EX_GOODS_INVENTORY_CHANGED_NOTIFY: ServerPacketId = ServerPacketId::new_ex(0xE2);
    const _EX_GOODS_INVENTORY_INFO: ServerPacketId = ServerPacketId::new_ex(0xE3);
    const _EX_GOODS_INVENTORY_RESULT: ServerPacketId = ServerPacketId::new_ex(0xE4);
    const _EX_2ND_PASSWORD_CHECK: ServerPacketId = ServerPacketId::new_ex(0xE5);
    const _EX_2ND_PASSWORD_VERIFY: ServerPacketId = ServerPacketId::new_ex(0xE6);
    const _EX_2ND_PASSWORD_ACK: ServerPacketId = ServerPacketId::new_ex(0xE7);
    const _EX_SAY2_FAIL: ServerPacketId = ServerPacketId::new_ex(0xE8);
}

#[repr(u16)]
#[derive(Clone, Debug, EnumDiscriminants, Event, From, Reflect)]
#[strum_discriminants(name(GameServerPacketKind))]
#[strum_discriminants(derive(Display, Reflect))]
pub enum GameServerPacket {
    ActionFail(ActionFail),
    Attack(Attack),
    AttackStanceStart(AttackStanceStart),
    AttackStanceStop(AttackStanceStop),
    ChangeMoveType(ChangeMoveType),
    ChangeWaitType(ChangeWaitType),
    CharInfo(CharInfo),
    CharSelectionInfo(CharSelectionInfo),
    CharacterCreationFailed(CharacterCreationFailed),
    CharacterCreationSuccess(CharacterCreationSuccess),
    CharacterDeletionFailed(CharacterDeletionFailed),
    CharacterDeletionSuccess(CharacterDeletionSuccess),
    CharacterSelected(CharacterSelected),
    CreatureSay(CreatureSay),
    DeleteObject(DeleteObject),
    Die(Die),
    DoorStatusUpdate(DoorStatusUpdate),
    DropItem(DropItem),
    EtcStatusUpdate(EtcStatusUpdate),
    ExBasicActionList(ExBasicActionList),
    ExBrExtraUserInfo(ExBrExtraUserInfo),
    ExRotation(ExRotation),
    InventoryUpdate(InventoryUpdate),
    ItemList(ItemList),
    KeyPacket(KeyPacket),
    LogoutOk(LogoutOk),
    MagicSkillLaunched(MagicSkillLaunched),
    MagicSkillCanceled(MagicSkillCanceled),
    MagicSkillUse(MagicSkillUse),
    MoveToLocation(MoveToLocation),
    MoveToPawn(MoveToPawn),
    MultisellList(MultisellList),
    NetPing(NetPing),
    NpcHtmlMessage(NpcHtmlMessage),
    NpcInfo(NpcInfo),
    ResponseCharCreateMenu(ResponseCharCreateMenu),
    RestartResponse(Restart),
    Revive(Revive),
    SSQInfo(SSQInfo),
    SelectTarget(SelectTarget),
    SetupGauge(SetupGauge),
    ShowMap(ShowMap),
    SkillList(SkillList),
    SocialAction(SocialAction),
    SpawnItem(SpawnItem),
    StaticObjectInfo(StaticObjectInfo),
    StatusUpdate(StatusUpdate),
    StopMove(StopMove),
    SystemMessage(SystemMessage),
    TargetUnselected(TargetUnselected),
    TeleportToLocation(TeleportToLocation),
    UserInfo(UserInfo),
    ValidateLocation(ValidateLocation),
    PlaySound(PlaySound),
    GetItem(GetItem),
    ShortcutRegistered(ShortcutRegistered),
    ShortcutInit(ShortcutInit),
    AbnormalStatusUpdate(AbnormalStatusUpdate),
    ResponseAutoShots(ResponseAutoShots),
}
impl Default for GameServerPacket {
    fn default() -> Self {
        GameServerPacket::from(LogoutOk)
    }
}

l2r_core::impl_buffer!(
    GameServerPacket,
    ActionFail,
    Attack,
    AttackStanceStart,
    AttackStanceStop,
    ChangeMoveType,
    ChangeWaitType,
    CharInfo,
    CharSelectionInfo,
    CharacterCreationFailed,
    CharacterCreationSuccess,
    CharacterDeletionFailed,
    CharacterDeletionSuccess,
    CharacterSelected,
    CreatureSay,
    DeleteObject,
    Die,
    DoorStatusUpdate,
    DropItem,
    EtcStatusUpdate,
    ExBasicActionList,
    ExBrExtraUserInfo,
    ExRotation,
    GetItem,
    InventoryUpdate,
    ItemList,
    KeyPacket,
    LogoutOk,
    MagicSkillLaunched,
    MagicSkillCanceled,
    MagicSkillUse,
    MoveToLocation,
    MoveToPawn,
    MultisellList,
    NetPing,
    NpcHtmlMessage,
    NpcInfo,
    ResponseCharCreateMenu,
    RestartResponse,
    Revive,
    SSQInfo,
    SelectTarget,
    SetupGauge,
    ShowMap,
    SocialAction,
    SkillList,
    SpawnItem,
    StaticObjectInfo,
    StatusUpdate,
    StopMove,
    SystemMessage,
    TargetUnselected,
    TeleportToLocation,
    UserInfo,
    ValidateLocation,
    PlaySound,
    ShortcutRegistered,
    ShortcutInit,
    AbnormalStatusUpdate,
    ResponseAutoShots
);

#[derive(Clone, Debug, Deref, DerefMut, Event, From, Reflect)]
pub struct GameServerPackets(Vec<GameServerPacket>);

pub struct GameServerPacketPlugin;

impl Plugin for GameServerPacketPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameServerPacket>();

        app.register_type::<GameServerPackets>()
            .register_type::<GameServerPacket>()
            .register_type::<BroadcastScope>()
            .register_type::<ActionFail>()
            .register_type::<Attack>()
            .register_type::<AttackStanceStart>()
            .register_type::<AttackStanceStop>()
            .register_type::<ChangeMoveType>()
            .register_type::<ChangeWaitType>()
            .register_type::<CharInfo>()
            .register_type::<CharSelectionInfo>()
            .register_type::<CharacterCreationFailed>()
            .register_type::<CharacterCreationSuccess>()
            .register_type::<CharacterDeletionFailed>()
            .register_type::<CharacterDeletionSuccess>()
            .register_type::<CharacterSelected>()
            .register_type::<CreatureSay>()
            .register_type::<DeleteObject>()
            .register_type::<Die>()
            .register_type::<DropItem>()
            .register_type::<EtcStatusUpdate>()
            .register_type::<ExBasicActionList>()
            .register_type::<ExBrExtraUserInfo>()
            .register_type::<ExRotation>()
            .register_type::<InventoryUpdate>()
            .register_type::<ItemList>()
            .register_type::<KeyPacket>()
            .register_type::<LogoutOk>()
            .register_type::<MagicSkillLaunched>()
            .register_type::<MagicSkillCanceled>()
            .register_type::<MagicSkillUse>()
            .register_type::<MoveToLocation>()
            .register_type::<MoveToPawn>()
            .register_type::<MultisellList>()
            .register_type::<NetPing>()
            .register_type::<NpcHtmlMessage>()
            .register_type::<NpcInfo>()
            .register_type::<ResponseCharCreateMenu>()
            .register_type::<PlaySound>()
            .register_type::<Restart>()
            .register_type::<Revive>()
            .register_type::<SSQInfo>()
            .register_type::<SelectTarget>()
            .register_type::<SetupGauge>()
            .register_type::<SetupGaugeColor>()
            .register_type::<ShowMap>()
            .register_type::<SocialAction>()
            .register_type::<Social>()
            .register_type::<SpawnItem>()
            .register_type::<StatusUpdate>()
            .register_type::<StatusUpdateField>()
            .register_type::<StatusUpdateFields>()
            .register_type::<StatusUpdateKind>()
            .register_type::<StopMove>()
            .register_type::<SystemMessage>()
            .register_type::<TargetUnselected>()
            .register_type::<TeleportToLocation>()
            .register_type::<UserInfo>()
            .register_type::<ValidateLocation>()
            .register_type::<ShortcutRegistered>()
            .register_type::<ShortcutInit>()
            .register_type::<AbnormalStatusUpdate>()
            .register_type::<ShotState>()
            .register_type::<ResponseAutoShots>();
    }
}
