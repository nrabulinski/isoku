#![allow(dead_code)]
use super::{Id, OsuEncode};
use crate::{token::Token, Channel, Match};
use std::sync::atomic::Ordering;

macro_rules! guess_size {
    ($val:expr) => { $val.encoded_size() };
    ($a:expr, $($b:expr),+) => { $a.encoded_size() + guess_size!($($b),+) };
}

macro_rules! build_packet {
    ($id:expr => $($val:expr),+) => { {
        let mut data = Vec::with_capacity(guess_size!($($val),+));
        $($val.encode(&mut data);)+
        let size = data.len() + 7;
        let mut buf = Vec::with_capacity(size);
        $id.encode(&mut buf);
        buf.push(0);
        (data.len() as i32).encode(&mut buf);
        buf.append(&mut data);
        buf
    } };

    ($id:expr) => {{
        let mut buf = Vec::with_capacity(7);
        $id.encode(&mut buf);
        buf.push(0);
        (0u32).encode(&mut buf);
        buf
    }}
}

// ---INFO---
#[inline]
pub fn silence_end(sec: u32) -> Vec<u8> { build_packet!(Id::SilenceEnd => sec) }

#[inline]
pub fn protocol_ver(version: u32) -> Vec<u8> { build_packet!(Id::ProtocolVersion => version) }

#[inline]
pub fn online_users(user_list: &[i32]) -> Vec<u8> {
    build_packet!(Id::UserPresenceBundle => user_list)
}

// ---LOGIN---
#[inline]
pub fn login_failed() -> Vec<u8> { build_packet!(Id::UserId => -1_i32) }

#[inline]
pub fn user_id(id: i32) -> Vec<u8> { build_packet!(Id::UserId => id) }

#[inline]
pub fn user_rank(_rank: u32) -> Vec<u8> { build_packet!(Id::SupporterGmt => 38u32) }

#[inline]
pub fn logout(token: &dyn Token) -> Vec<u8> { build_packet!(Id::UserLogout => token.id(), 0u8) }

// ---USER INFO---
#[inline]
pub fn user_panel(token: &dyn Token) -> Vec<u8> {
    build_packet!(Id::UserPanel =>
        token.id(), token.username(), 0i16, 16u8, 0f32, 0f32, 1u32)
}

#[inline]
pub async fn user_stats(token: &dyn Token) -> Vec<u8> {
    let stats = token.stats().await;
    let action_id = stats.action as u8;
    let action_text = &stats.action_text;
    let action_md5 = &stats.action_md5;
    let game_mode = stats.game_mode as u8;
    build_packet!(Id::UserStats =>
        token.id(),
        action_id,
        action_text,
        action_md5,
        stats.action_mods,
        game_mode,
        stats.beatmap_id,
        stats.ranked_score,
        stats.accuracy,
        stats.playcount,
        stats.total_score,
        stats.rank,
        stats.pp
    )
}

#[inline]
pub fn friend_list(users: &[i32]) -> Vec<u8> { build_packet!(Id::FriendsList => users) }

// ---CHAT---
#[inline]
pub fn channel_info_end() -> Vec<u8> { build_packet!(Id::ChannelInfoEnd => 0u32) }

#[inline]
pub async fn channel_info(channel: &Channel) -> Vec<u8> {
    let name = channel.name();
    let desc = &channel.desc;
    let users = channel.users.read().await.len() as u16;
    build_packet!(Id::ChannelInfo => name, desc, users)
}

#[inline]
pub fn channel_join_success(channel: &Channel) -> Vec<u8> {
    build_packet!(Id::ChannelJoinSuccess => channel.name())
}

#[inline]
pub fn channel_kicked(channel: &Channel) -> Vec<u8> {
    build_packet!(Id::ChannelKicked => channel.name())
}

#[inline]
pub fn send_message(from: &dyn Token, to: &str, content: &str) -> Vec<u8> {
    build_packet!(Id::SendMessage => from.username(), content, to, from.id())
}

// ---MULTI---
#[inline]
async fn match_info(id: Id, m: &Match) -> Vec<u8> {
    let p = m.password.read().await;
    let password = if let Some(pass) = &*p { pass } else { "" };
    let match_name = &m.name.read().await;
    let beatmap_name = m.beatmap_name.read().await;
    let beatmap_id = m.beatmap_id.load(Ordering::SeqCst);
    let beatmap_md5 = m.beatmap_md5.read().await;
    let slot_ids = m.slot_ids().await;
    let host_id = m.host_id.read().await;
    build_packet!(id =>
        m.id,
        m.in_progress.load(Ordering::SeqCst) as u8,
        0u8,
        m.mods.load(Ordering::SeqCst),
        match_name,
        password,
        beatmap_name,
        beatmap_id,
        beatmap_md5,
        m.slot_statuses(),
        m.slot_teams(),
        slot_ids,
        host_id,
        m.game_mode.load(Ordering::SeqCst),
        m.scoring_type.load(Ordering::SeqCst),
        m.team_type.load(Ordering::SeqCst),
        m.freemod.load(Ordering::SeqCst) as u8,
        m.slot_mods(),
        0u32
    )
}

#[inline]
pub async fn create_match(m: &Match) -> Vec<u8> { match_info(Id::CreateMatch, m).await }

#[inline]
pub async fn match_join_success(m: &Match) -> Vec<u8> { match_info(Id::MatchJoinSuccess, m).await }

#[inline]
pub fn match_join_fail() -> Vec<u8> { build_packet!(Id::MatchJoinFail) }

#[inline]
pub fn match_transfer_host() -> Vec<u8> { build_packet!(Id::ServerMatchTransferHost) }

#[inline]
pub async fn update_match(m: &Match) -> Vec<u8> { match_info(Id::UpdateMatch, m).await }

// ---UTILS---
#[inline]
pub fn notification(text: &str) -> Vec<u8> { build_packet!(Id::Notification => text) }

#[inline]
pub fn jumpscare(text: &str) -> Vec<u8> { build_packet!(Id::Jumpscare => text) }
