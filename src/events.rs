use std::sync::Arc;
use super::{Glob, osu};
use osu::OsuData;
use osu::token::Token;

pub fn send_public_message(data: &[u8], token: &Token, glob: &Glob) {
    let (u, message, to) = {
        let (len, u) = String::decode(data);
        let (l, message) = String::decode(&data[len..]);
        let len = len + l;
        let (_, to) = String::decode(&data[len..]);
        (u, message, to)
    };

    if to == "#multiplayer" || to == "#spectator" {
        // TODO: Handle multiplayer and spectator chatrooms
        return
    }

    println!("SEND PUBLIC MESSAGE {},{},{}", u,message,to);

    let channel =
        if let Some(channel) = glob.channel_list.get(&to) { channel }
        else {
            return;
        };
    println!("{:?}", channel);
    let channel_users = channel.users();

    let packet = osu::packets::server::send_message(token, to, message);

    channel_users.iter()
        .filter(|t| t.id() != token.id())
        .for_each(|t| { println!("enquing for {:?}", t); t.enqueue(&packet) });
}

pub fn logout(token: &str, glob: &Glob) {
    let user =
        if let Some(token) = glob.token_list.remove(token) { token }
        else {
            return;
        };

    let channels = user.joined_channels();
    for channel in channels.iter() {
        channel.upgrade().unwrap().remove_client(&user);
    }
    println!("AFTER LOGOUT:\n{:?}\n{:?}", glob.token_list.entries(), glob.channel_list.entries());
}

pub fn channel_join(data: &[u8], token: Arc<Token>, glob: &Glob) {
    let (_, channel_name) = String::decode(data);

    if let Some(channel) = glob.channel_list.get(&channel_name) {
        token.join_channel(Arc::downgrade(&channel));
        channel.add_client(token);
    };
}

pub fn user_stats_request<'a>(data: &'a[u8], token: &Token, glob: &Glob) {
    use osu::packets::server::user_stats;

    let (_, users): (_, &'a[i32]) = OsuData::decode(data);
    println!("user_stats_request\n{:?}\n", users);

    glob.token_list.entries().into_iter()
        .filter(|t| users.contains(&(t.id() as i32)) && t.id() != token.id())
        .for_each(|t| token.enqueue(&user_stats(&t)));
}

pub fn user_panel_request<'a>(data: &'a[u8], token: &Token, glob: &Glob) {
    use osu::packets::server::user_panel;

    let (_, users): (_, &'a[i32]) = OsuData::decode(data);
    println!("user_panel_request\n{:?}\n", users);

    glob.token_list.entries().into_iter()
        .filter(|t| users.contains(&(t.id() as i32)))
        .for_each(|t| token.enqueue(&user_panel(&t)));
}