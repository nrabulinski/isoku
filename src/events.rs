use std::sync::Arc;
use super::{Glob, osu};
use osu::OsuData;
use osu::token::Token;
use super::Cursor;

pub fn send_public_message(data: &mut Cursor<'_>, token: &Arc<Token>, glob: &Glob) {
    let (message, to) = {
        let _ = String::decode(data);
        let message = String::decode(data);
        let to = String::decode(data);
        (message, to)
    };

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
    let (message, to) = {
        String::decode(data);
        (String::decode(data), String::decode(data))
    };

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
    let channel_name = String::decode(data);

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
    use osu::packets::server::user_stats;

    let users: &'a[i32] = OsuData::decode(data);
    println!("user_stats_request\n{:?}\n", users);

    glob.token_list.entries().into_iter()
        .filter(|t| users.contains(&(t.id() as i32)) && t.id() != token.id())
        .for_each(|t| token.enqueue(&user_stats(&t)));
}

pub fn user_panel_request<'a>(data: &mut Cursor<'a>, token: &Token, glob: &Glob) {
    use osu::packets::server::user_panel;

    let users: &'a[i32] = OsuData::decode(data);
    println!("user_panel_request\n{:?}\n", users);

    glob.token_list.entries().into_iter()
        .filter(|t| users.contains(&(t.id() as i32)))
        .for_each(|t| token.enqueue(&user_panel(&t)));
}