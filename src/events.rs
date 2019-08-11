use std::sync::Arc;
use super::{Glob, osu};
use osu::token::Token;
use crate::bytes::Cursor;

//I would be fine without this macro but I wrote it for two main reasons:
//1. Macros are cool and I like writing them
//2. To create a convention and make it even more clear how the data for said event is structured
macro_rules! event_data {
    ( $data:expr; $type:ty ) => { $data.get::<$type>().unwrap() };
    ( $data:expr; $( $type:ty ),* ) => { ( $( $data.get::<$type>().unwrap() ),* ) };
}

pub fn change_action(data: &mut Cursor, _token: &Token, _glob: &Glob) {
    let data = event_data!(data; u8, String, String, u32, u8, i32); //(id, text, md5, mods, gm, beatmap_id)
    //TODO; actually update the user, I guess
    //also I was thinking maybe below certain user count
    //I might send the presence to everyone so that it gets automatically updated?
    trace!("{:?} changed their action to {:?}", _token.token(), data);
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

    let packet = osu::packets::server::send_message(token, to, message);

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

    let packet = osu::packets::server::send_message(token, to.username(), message);
    to.enqueue(&packet);
}

pub fn channel_join(data: &mut Cursor<'_>, token: Arc<Token>, glob: &Glob) {
    let channel_name = event_data!(data; String);

    if let Some(channel) = glob.channel_list.get(&channel_name) {
        token.join_channel(Arc::downgrade(&channel));
        if channel.add_client(token.clone()) {
            token.enqueue(&osu::packets::server::channel_join_success(channel.name()));
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
        .for_each(|t| token.enqueue(&osu::packets::server::user_stats(&t)));
}

pub fn channel_part(data: &mut Cursor, token: &Arc<Token>, glob: &Glob) {
    //TODO
}

pub fn user_panel_request<'a>(data: &mut Cursor<'a>, token: &Token, glob: &Glob) {
    let users = event_data!(data; &'a[i32]);
    println!("user_panel_request\n{:?}\n", users);

    glob.token_list.entries().into_iter()
        .filter(|t| users.contains(&(t.id() as i32)))
        .for_each(|t| token.enqueue(&osu::packets::server::user_panel(&t)));
}