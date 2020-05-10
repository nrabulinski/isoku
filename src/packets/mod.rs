use std::convert::TryFrom;
pub mod encoding;
pub mod server;
pub use encoding::OsuEncode;

#[macro_use]
macro_rules! count_items {
    () => { 0 };
    ($a:ident) => { 1 };
    ($a:ident, $($b:ident),+) => { 1 + count_items!($($b),+) }
}

#[macro_use]
macro_rules! enum_try_from {
    (
        #[repr($t:ident)]
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($field:ident = $val:expr),* $(,)?
        }
    ) => {
       #[repr($t)]
       $(#[$meta])*
       $vis enum $name {
           $($field = $val),*
       }

       impl std::convert::TryFrom<$t> for $name {
           type Error = ();
           fn try_from(value: $t) -> Result<$name, ()> {
               match value {
                   $(
                       $val => Ok($name::$field),
                   )*
                   _ => Err(())
               }
           }
       }

        impl $crate::packets::OsuEncode for $name {
            fn encoded_size(&self) -> usize { std::mem::size_of::<$t>() }

            fn encode(&self, buf: &mut Vec<u8>) {
                (*self as $t).encode(buf);
            }

            fn decode(buf: &[u8]) -> Result<(&Self, usize), ()> {
                //use $crate::packets::OsuEncode;
                let (&val, off) = <$t>::decode(buf)?;
                use std::convert::TryFrom;
                Self::try_from(val)?;
                Ok((
                    unsafe { &*(buf.as_ptr() as *const Self) },
                    off
                ))
            }
        }
    };

    (
        #[repr($t:ident)]
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($field:ident),* $(,)?
        }
    ) => {
        #[repr($t)]
        $(#[$meta])*
        $vis enum $name {
            $($field),*
        }

        impl std::convert::TryFrom<$t> for $name {
            type Error = ();
            fn try_from(value: $t) -> Result<$name, ()> {
                const MAX: $t = count_items!($($field),*) - 1;
                if value > MAX { Err(()) }
                else {
                    Ok(unsafe {
                        std::mem::transmute::<$t, $name>(value)
                    })
                }
            }
        }

        impl $crate::packets::OsuEncode for $name {
            fn encoded_size(&self) -> usize { std::mem::size_of::<$t>() }

            fn encode(&self, buf: &mut Vec<u8>) {
                (*self as $t).encode(buf);
            }

            fn decode(buf: &[u8]) -> Result<(&Self, usize), ()> {
                //use $crate::packets::OsuEncode;
                let (&val, off) = <$t>::decode(buf)?;
                use std::convert::TryFrom;
                Self::try_from(val)?;
                Ok((
                    unsafe { &*(buf.as_ptr() as *const Self) },
                    off
                ))
            }
        }
    }
}

enum_try_from!(
    #[repr(u16)]
    #[derive(Debug, Clone, Copy)]
    #[allow(dead_code, clippy::enum_variant_names)]
    pub enum Id {
        ChangeAction = 0,
        SendPublicMessage = 1,
        Logout = 2,
        RequestStatusUpdate = 3,
        Pong = 4,
        UserId = 5,
        //commandError = 6,
        SendMessage = 7,
        //ping = 8,
        HandleIrcUsernameChange = 9,
        HandleIrcQuit = 10,
        UserStats = 11,
        UserLogout = 12,
        SpectatorJoined = 13,
        SpectatorLeft = 14,
        ServerSpectateFrames = 15,
        StartSpectating = 16,
        StopSpectating = 17,
        SpectateFrames = 18,
        //versionUpdate = 19,
        CantSpectate = 21,
        SpectatorCantSpectate = 22,
        GetAttention = 23,
        Notification = 24,
        SendPrivateMessage = 25,
        UpdateMatch = 26,
        NewMatch = 27,
        DisposeMatch = 28,
        PartLobby = 29,
        JoinLobby = 30,
        CreateMatch = 31,
        JoinMatch = 32,
        PartMatch = 33,
        //lobbyJoinObsolete = 34,
        //lobbyPartObsolete = 35,
        MatchJoinSuccess = 36,
        MatchJoinFail = 37,
        MatchReady = 39,
        MatchLock = 40,
        MatchChangeSettings = 41,
        FellowSpectatorJoined = 42,
        FellowSpectatorLeft = 43,
        ClientMatchStart = 44,
        AllPlayersLoaded = 45,
        ServerMatchStart = 46,
        ClientMatchScoreUpdate = 47,
        ServerMatchScoreUpdate = 48,
        ClientMatchComplete = 49,
        ServerMatchTransferHost = 50,
        MatchChangeMods = 51,
        MatchLoadComplete = 52,
        MatchAllPlayersLoaded = 53,
        MatchNoBeatmap = 54,
        MatchNotReady = 55,
        MatchFailed = 56,
        MatchPlayerFailed = 57,
        ServerMatchComplete = 58,
        MatchHasBeatmap = 59,
        MatchSkipRequest = 60,
        MatchSkip = 61,
        //unauthorised = 62,
        ChannelJoin = 63,
        ChannelJoinSuccess = 64,
        ChannelInfo = 65,
        ChannelKicked = 66,
        //channelAvailableAutojoin = 67,
        //beatmapInfoReply = 69,
        ClientMatchTransferHost = 70,
        SupporterGmt = 71,
        FriendsList = 72,
        FriendAdd = 73,
        FriendRemove = 74,
        ProtocolVersion = 75,
        MainMenuIcon = 76,
        MatchChangeTeam = 77,
        ChannelPart = 78,
        ReceiveUpdates = 79,
        //topBotnet = 80,
        MatchPlayerSkipped = 81,
        SetAwayMessage = 82,
        UserPanel = 83,
        //IRCOnly = 84,
        UserStatsRequest = 85,
        Restart = 86,
        ClientInvite = 87,
        ServerInvite = 88,
        ChannelInfoEnd = 89,
        ClientMatchChangePassword = 90,
        ServerMatchChangePassword = 91,
        SilenceEnd = 92,
        SpecialMatchInfoRequest = 93,
        UserSilenced = 94,
        //userPresenceSingle = 95,
        UserPresenceBundle = 96,
        UserPanelRequest = 97,
        //userPmBlocked = 100,
        //targetIsSilenced = 101,
        //versionUpdateForced = 102,
        //switchServer = 103,
        AccountRestricted = 104,
        Jumpscare = 105,
        MatchAbort = 106,
        SwitchTourneyServer = 107,
        SpecialJoinMatchChannel = 108,
        SpecialLeaveMatchChannel = 109,
        Unknown = 999,
    }
);

pub fn parse_packet(buf: &[u8]) -> Result<(Id, usize), ()> {
    let (&id, _) = u16::decode(buf)?;
    let buf = &buf[3..];
    let (&len, _) = u32::decode(buf)?;
    if (&buf[4..]).len() < len as usize {
        println!("{:?} {} {}", id, &buf[4..].len(), len);
        return Err(());
    }
    Ok((Id::try_from(id).unwrap_or(Id::Unknown), len as usize))
}
