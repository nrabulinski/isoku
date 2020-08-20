use crate::{packets::server::send_message as message_packet, Glob, Token};

pub async fn handle_command(cmd: &str, token: &dyn Token, glob: &Glob) -> Result<(), String> {
    let mut cmd = cmd.split(' ');
    let command = cmd.next().unwrap();
    // let i = cmd.find(' ').unwrap_or(cmd.len() - 1);
    // let command = &cmd[..i];
    // let arg = &cmd[i + 1..];
    match command {
        "echo" => {
            token
                .enqueue_vec(message_packet(
                    glob.bot.as_ref(),
                    token.username(),
                    &cmd.collect::<Vec<&str>>().join(" "),
                ))
                .await;
            Ok(())
        }
        "kick" => {
            let user = cmd.next().unwrap();
            let t = glob
                .token_list
                .read()
                .await
                .values()
                .find(|t| t.username() == user)
                .ok_or_else(|| format!("{} is not online", user))?;
            let mut msg = format!("You have been kicked by {}!", token.username());
            let reason = cmd.collect::<Vec<&str>>().join(" ");
            if !reason.is_empty() {
                msg += &format!("\nReason: \"{}\"", reason);
            }
            Ok(())
        }
        _ => Err(format!("No such command \"{}\"! Try: !help", command)),
    }
}

pub async fn send_message(content: &str, channel: &str, glob: &Glob) -> Result<(), String> {
    let channel = glob
        .channel_list
        .read()
        .await
        .get(channel)
        .ok_or_else(|| format!("No channel named {}", channel))?
        .clone();
    let packet = message_packet(glob.bot.as_ref(), channel.name(), content);
    for c in channel.users.read().await.iter() {
        c.enqueue(&packet).await;
    }
    Ok(())
}
