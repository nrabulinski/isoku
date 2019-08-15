use std::sync::Arc;
use super::{Glob, osu};
use osu::token::Token;
use crate::bytes::Cursor;
use osu::packets::server as packets;

//I would be fine without this macro but I wrote it for two main reasons:
//1. Macros are cool and I like writing them
//2. To create a convention and make it even more clear how the data for said event is structured
macro_rules! event_data {
    ( $data:expr; $type:ty ) => { $data.get::<$type>().unwrap() };
    ( $data:expr; $( $type:ty ),* ) => { ( $( $data.get::<$type>().unwrap() ),* ) };
}

pub fn change_action(data: &mut Cursor, token: &Token, glob: &Glob) {
    let data = event_data!(data; u8, String, String, u32, u8, i32); //(id, text, md5, mods, gm, beatmap_id)
    //TODO; actually update the user, I guess
    //also I was thinking maybe below certain user count
    //I might send the presence to everyone so that it gets automatically updated?
    trace!("{:?} changed their action to {:?}", token.token(), data);

    glob.token_list.enqueue_all(&packets::user_panel(token));
}

pub fn send_public_message(data: &mut Cursor<'_>, token: &Arc<Token>, glob: &Glob) {
    let (_, message, to) = event_data!(data; String, String, String);

    if to == "#multiplayer" || to == "#spectator" {
        // TODO: Handle multiplayer and spectator chatrooms
        return
    }

    println!("SEND PUBLIC MESSAGE {},{}", message,to);

    let channel =
        if let Some(channel) = glob.channel_list.get(&to) { channel }
        else {
            return;
        };

    if !channel.has_client(token) {
        warn!("{:?} tried sending message to {:?} before joining it", token.token(), to);
        return;
    }
    let channel_users = channel.users();

    let packet = packets::send_message(token, to, message);

    channel_users.iter()
        .filter(|t| t.id() != token.id())
        .for_each(|t| { println!("enquing for {:?}", t); t.enqueue(&packet) });
}

pub fn logout(token: &str, glob: &Glob) {
    let user = match glob.token_list.remove(token) {
        Some(token) => token,
        _ => {
            warn!("{:?} tried to log out not being logged in", token);
            return;
        }
    };

    let channels = user.joined_channels();
    for channel in channels.iter() {
        channel.upgrade().unwrap().remove_client(&user);
    }
    println!("AFTER LOGOUT:\n{:?}\n{:?}", glob.token_list.entries(), glob.channel_list.entries());
    glob.token_list.enqueue_all(&packets::logout(&user));
}

pub fn send_private_message(data: &mut Cursor<'_>, token: &Token, glob: &Glob) {
    let (_, message, to, _) = event_data!(data; String, String, String, String);

    let to = match glob.token_list.get_username(&to) {
        Some(user) => user,
        None => {
            //TODO: Notify about messaging an inactive user
            return;
        }
    };

    let packet = packets::send_message(token, to.username(), message);
    to.enqueue(&packet);
}

pub fn join_lobby(token: &Token, glob: &Glob) {
    glob.match_list.entries()
        .into_iter()
        .for_each(|m| token.enqueue(&packets::create_match(&m, true)));
}

#[derive(Debug)]
pub struct MatchSettings {
    id: u16, in_progress: u8,
    mods: u32, name: String,
    password: String, beatmap_name: String,
    beatmap_id: i32, beatmap_md5: String,
    host_user_id: i32, game_mode: u8, score_type: u8,
    team_type: u8, freemod: u8
}

fn match_data(data: &mut Cursor) -> MatchSettings {
    let (id, in_progress, _, mods, name, password, beatmap_name, beatmap_id, beatmap_md5) =
        event_data!(data; u16, u8, u8, u32, String, String, String, i32, String);

    let mut free_slots = 0;
    for _ in 0..16 { if data.get::<u8>().unwrap() == 1 { free_slots += 1 } }
    //skip not used slot team
    data.advance(16);

    let (host_user_id, game_mode, score_type, team_type, freemod) =
        event_data!(data; i32, u8, u8, u8, u8);

    let data = MatchSettings {
        id, in_progress, mods, name, password, beatmap_name, beatmap_id, beatmap_md5,
        host_user_id, game_mode, score_type, team_type, freemod
    };
    
    trace!("Parsed match settings: {:?}", data);
    trace!("Number of free slots: {}", free_slots);
    data
}

pub fn create_match(data: &mut Cursor, token: &Token, glob: &Glob) {
    let MatchSettings { name, password, beatmap_id, beatmap_name, beatmap_md5, game_mode,
        id: _, in_progress: _, mods: _, host_user_id: _, score_type: _, team_type: _, freemod: _ } =
        match_data(data);

    let conn = glob.db_pool.get().unwrap();
    let multi = glob.match_list.create_match(name, password, beatmap_id, beatmap_name, beatmap_md5, game_mode, token.id(), &conn);
    let channel = glob.channel_list.add_channel(format!("#multi_{}", multi.id()), "".to_string(), false);
    
    token.enqueue(&packets::match_join_success(&multi));
}

pub fn channel_join(data: &mut Cursor<'_>, token: Arc<Token>, glob: &Glob) {
    let channel_name = event_data!(data; String);

    if let Some(channel) = glob.channel_list.get(&channel_name) {
        token.join_channel(Arc::downgrade(&channel));
        if channel.add_client(token.clone()) {
            token.enqueue(&packets::channel_join_success(channel.name()));
        } else {
            warn!("{:?} couldn't join {:?}", token.token(), channel_name);
        }
    } else {
        warn!("{:?} tried to join inexistent channel {:?}", token.token(), channel_name);
    }
}

pub fn user_stats_request<'a>(data: &mut Cursor<'a>, token: &Token, glob: &Glob) {
    let users = event_data!(data; &'a[i32]);
    println!("user_stats_request\n{:?}\n", users);

    glob.token_list.entries().into_iter()
        .filter(|t| users.contains(&(t.id() as i32)) && t.id() != token.id())
        .for_each(|t| token.enqueue(&packets::user_stats(&t)));
}

pub fn channel_part(data: &mut Cursor, token: &Arc<Token>, glob: &Glob) {
    //TODO
}

pub fn user_panel_request<'a>(data: &mut Cursor<'a>, token: &Token, glob: &Glob) {
    let users = event_data!(data; &'a[i32]);
    println!("user_panel_request\n{:?}\n", users);

    glob.token_list.entries().into_iter()
        .filter(|t| users.contains(&(t.id() as i32)))
        .for_each(|t| token.enqueue(&packets::user_panel(&t)));
}