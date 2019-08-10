pub mod server {
    use crate::bytes::{Bytes, AsBuf};
    use crate::osu::{token::Token, channel::Channel};

    #[allow(non_camel_case_types)]
    enum ID {
        USER_ID = 5,
        COMMAND_ERROR = 6,
        SEND_MESSAGE = 7,
        PING = 8,
        HANDLE_IRC_USERNAME_CHANGE = 9,
        HANDLE_IRC_QUIT = 10,
        USER_STATS = 11,
        USER_LOGOUT = 12,
        SPECTATOR_JOINED = 13,
        SPECTATOR_LEFT = 14,
        SPECTATE_FRAMES = 15,
        VERSION_UPDATE = 19,
        SPECTATOR_CANT_SPECTATE = 22,
        GET_ATTENTION = 23,
        NOTIFICATION = 24,
        UPDATE_MATCH = 26,
        NEW_MATCH = 27,
        DISPOSE_MATCH = 28,
        //LOBBY_JOIN_OBSOLETE = 34,
        //LOBBY_PART_OBSOLETE = 35,
        MATCH_JOIN_SUCCESS = 36,
        MATCH_JOIN_FAIL = 37,
        FELLOW_SPECTATOR_JOINED = 42,
        FELLOW_SPECTATOR_LEFT = 43,
        ALL_PLAYERS_LOADED = 45,
        MATCH_START = 46,
        MATCH_SCORE_UPDATE = 48,
        MATCH_TRANSFER_HOST = 50,
        MATCH_ALL_PLAYERS_LOADED = 53,
        MATCH_PLAYER_FAILED = 57,
        MATCH_COMPLETE = 58,
        MATCH_SKIP = 61,
        UNAUTHORISED = 62,
        CHANNEL_JOIN_SUCCESS = 64,
        CHANNEL_INFO = 65,
        CHANNEL_KICKED = 66,
        CHANNEL_AVAILABLE_AUTOJOIN = 67,
        BEATMAP_INFO_REPLY = 69,
        SUPPORTER_GMT = 71,
        FRIENDS_LIST = 72,
        PROTOCOL_VERSION = 75,
        MAIN_MENU_ICON = 76,
        TOP_BOTNET = 80,
        MATCH_PLAYER_SKIPPED = 81,
        USER_PANEL = 83,
        //_I_R_C_ONLY = 84,
        RESTART = 86,
        INVITE = 88,
        CHANNEL_INFO_END = 89,
        MATCH_CHANGE_PASSWORD = 91,
        SILENCE_END = 92,
        USER_SILENCED = 94,
        USER_PRESENCE_SINGLE = 95,
        USER_PRESENCE_BUNDLE = 96,
        USER_PM_BLOCKED = 100,
        TARGET_IS_SILENCED = 101,
        VERSION_UPDATE_FORCED = 102,
        SWITCH_SERVER = 103,
        ACCOUNT_RESTRICTED = 104,
        JUMPSCARE = 105,
        SWITCH_TOURNEY_SERVER = 107,
    }

    fn build_packet(id: ID, data: impl AsBuf) -> Vec<u8> {
        let mut buf = Vec::with_capacity(7);
        buf.put(id as i16);
        buf.put(0_u8);
        buf.reserve(4 + data.size());
        buf.put(data.size() as i32);
        buf.put(data);
        buf.to_vec()
    }

    /* LOGIN */
    pub fn login_failed() -> Vec<u8> {
        build_packet(ID::USER_ID, -1_i32)
    }

    pub fn login_banned() -> Vec<u8> {
        [login_failed(), notification("You have been banned")].concat()
    }

    pub fn login_error() -> Vec<u8> {
        build_packet(ID::USER_ID, -5_i32)
    }

    pub fn user_id(id: u32) -> Vec<u8> {
        build_packet(ID::USER_ID, id)
    }

    pub fn user_rank(rank: u32) -> Vec<u8> {
        build_packet(ID::SUPPORTER_GMT, 38_u32)
    }

    pub fn user_panel(token: &Token) -> Vec<u8> {
        let data = {
            let username = token.username();
            let mut buf = Vec::with_capacity(19 + username.size());
            buf.put(token.id() as i32);
            buf.put(username);
            buf.put(0_u8);
            buf.put(0_u8);
            buf.put(38_u8);
            let location = token.location();
            buf.put(location[0]);
            buf.put(location[1]);
            buf.put(1.0_f32);
            buf
        };
        build_packet(ID::USER_PANEL, data)
    }

    pub fn user_stats(token: &Token) -> Vec<u8> {
        let data = {
            let action_text = "".to_string();
            let action_md5 = "".to_string();
            let mut buf = Vec::with_capacity(44 + action_text.size() + action_md5.size());
            buf.put(token.id());
            buf.put(0);          // action id
            buf.put(action_text);//"Beta-testing".to_string().encode());
            buf.put(action_md5); //buf.put();
            buf.put(0_i32);      // action mods?
            buf.put(0_u8);          // game mode
            buf.put(0_i32);      // beatmap id
            buf.put(1_u64);      // ranked score
            buf.put(1.0_f32);  // accuracy
            buf.put(1_u32);      // play count
            buf.put(1_u64);      // total score
            buf.put(1_u32);      // global rank
            buf.put(727_u16);      // pp
            buf
        };
        build_packet(ID::USER_STATS, data)
    }

    pub fn friend_list(users: &[i32]) -> Vec<u8> {
        build_packet(ID::FRIENDS_LIST, users)
    }

    pub fn silence_end(sec: u32) -> Vec<u8> {
        build_packet(ID::SILENCE_END, sec)
    }

    pub fn protocol_ver() -> Vec<u8> {
        build_packet(ID::PROTOCOL_VERSION, 19_u32)
    }

    pub fn online_users(user_list: &[i32]) -> Vec<u8> {
        build_packet(ID::USER_PRESENCE_BUNDLE, user_list)
    }

    /* CHAT */
    pub fn send_message(from: &Token, to: String, message: String) -> Vec<u8> {
        let data = {
            let user = from.username();
            let mut buf = Vec::with_capacity(user.size() + to.size() + message.size() + 4);
            buf.put(user);
            buf.put(message);
            buf.put(to);
            buf.put(from.id());
            buf
        };
        build_packet(ID::SEND_MESSAGE, data)
    }

    pub fn channel_info(channel: &Channel) -> Vec<u8> {
        let data = {
            let name = channel.name().to_string();
            let desc = channel.desc().to_string();
            let users = channel.users_len();
            let mut buf = Vec::with_capacity(name.size() + desc.size() + 2);
            buf.put(name);
            buf.put(desc);
            buf.put(users);
            buf
        };
        build_packet(ID::CHANNEL_INFO, data)
    }

    pub fn channel_info_end() -> Vec<u8> {
        build_packet(ID::CHANNEL_INFO_END, 0_u32)
    }

    pub fn channel_join_success(name: &str) -> Vec<u8> {
        build_packet(ID::CHANNEL_JOIN_SUCCESS, name.to_string())
    }

    /* UTILS */
    pub fn notification(text: &str) -> Vec<u8> {
        build_packet(ID::NOTIFICATION, text.to_string())
    }

    pub fn menu_icon(url: &str) -> Vec<u8> {
        build_packet(ID::MAIN_MENU_ICON, url.to_string())
    }
}

pub mod client {
    use crate::bytes::Cursor;

    #[allow(non_camel_case_types)]
    #[derive(Debug, enumn::N)]
    pub enum ID {
        CHANGE_ACTION = 0, //TODO
        SEND_PUBLIC_MESSAGE = 1,
        LOGOUT = 2,
        REQUEST_STATUS_UPDATE = 3,
        PONG = 4,
        START_SPECTATING = 16, //TODO
        STOP_SPECTATING = 17, //TODO
        SPECTATE_FRAMES = 18, //TODO
        ERROR_REPORT = 20, //TODO
        CANT_SPECTATE = 21, //TODO
        SEND_PRIVATE_MESSAGE = 25,
        PART_LOBBY = 29, //TODO
        JOIN_LOBBY = 30, //TODO
        CREATE_MATCH = 31, //TODO
        JOIN_MATCH = 32, //TODO
        PART_MATCH = 33, //TODO
        MATCH_READY = 39, //TODO
        MATCH_LOCK = 40, //TODO
        MATCH_CHANGE_SETTINGS = 41, //TODO
        MATCH_START = 44, //TODO
        ALL_PLAYERS_LOADED = 45, //TODO
        MATCH_SCORE_UPDATE = 47, //TODO
        MATCH_COMPLETE = 49, //TODO
        MATCH_CHANGE_MODS = 51, //TODO
        MATCH_LOAD_COMPLETE = 52, //TODO
        MATCH_NO_BEATMAP = 54, //TODO
        MATCH_NOT_READY = 55, //TODO
        MATCH_FAILED = 56, //TODO
        MATCH_HAS_BEATMAP = 59, //TODO
        MATCH_SKIP_REQUEST = 60, //TODO
        CHANNEL_JOIN = 63,
        //BEATMAP_INFO_REQUEST = 68, unknown?
        MATCH_TRANSFER_HOST = 70, //TODO
        FRIEND_ADD = 73, //TODO
        FRIEND_REMOVE = 74, //TODO
        MATCH_CHANGE_TEAM = 77, //TODO
        CHANNEL_PART = 78, //TODO
        RECEIVE_UPDATES = 79, //TODO
        SET_AWAY_MESSAGE = 82, //TODO
        //I_R_C_ONLY = 84,
        USER_STATS_REQUEST = 85,
        INVITE = 87, //TODO
        MATCH_CHANGE_PASSWORD = 90, //TODO
        SPECIAL_MATCH_INFO_REQUEST = 93, //TODO
        USER_PRESENCE_REQUEST = 97,
        USER_PRESENCE_REQUEST_ALL = 98, //TODO?
        USER_TOGGLE_BLOCK_NON_FRIEND_PM = 99, //TODO
        MATCH_ABORT = 106, //TODO
        SPECIAL_JOIN_MATCH_CHANNEL = 108, //TODO
        SPECIAL_LEAVE_MATCH_CHANNEL = 109, //TODO
        UNKNOWN = -1
    }

    pub fn parse_packet<'b>(buf: &mut Cursor<'b>) -> (ID, Cursor<'b>) {
        let id_raw: u16 = buf.get().unwrap();
        let id = ID::n(id_raw).unwrap_or_else(|| {eprintln!("Unknown id: {}",id_raw); ID::UNKNOWN});
        buf.advance(1);
        let len: u32 = buf.get().unwrap();
        if buf.remaining() < len as usize {
            error!("packet {} had length of {}, which was greater than the length of its data", id_raw, len);
            return (ID::UNKNOWN, Cursor::new(&[]));
        }
        let data = if len > 0 {
            Cursor::new(buf.read(len as usize).unwrap())
        } else { Cursor::new(&[]) };
        (id, data)
    }

    // pub fn channel_join(buf: &[u8]) -> String {
    //     String::decode(buf)
    // }
    // struct Packet {
    //     id: ID
    // }

    // impl From<&[u8]> for Packet {
    //     fn from(buf: &[u8]) -> Self {
            
    //     }
    // }
}