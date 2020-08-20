use crate::{bot::handle_command, event_data, packets::server::send_message, Glob, Token};
use std::sync::Arc;

#[event_data]
struct Message {
    _na: &str, // undefined field, probably always a single null byte?
    content: &str,
    to: &str,
}

async fn common<'a>(data: &'a [u8], token: &dyn Token, glob: &Glob) -> Result<Message<'a>, String> {
    let msg = Message::decode(data).map_err(|_| "Couldn't decode message".to_string())?;
    if msg.content.starts_with('!') && msg.content.len() > 1 {
        handle_command(&msg.content[1..], token, glob).await?;
    }
    Ok(msg)
}

pub async fn public(data: &[u8], token: &Arc<dyn Token>, glob: &Glob) -> Result<(), String> {
    let msg = common(data, token.as_ref(), glob).await?;
    let channel_name = match token.as_player() {
        Some(t) if msg.to == "#multiplayer" => {
            let multi = t.multi.lock().await;
            let multi = multi
                .as_ref()
                .ok_or_else(|| "You aren't in a multiplayer match".to_string())?
                .upgrade()
                .ok_or_else(|| "Multiplayer match has already ended")?;
            format!("#multi_{}", multi.id)
        }
        _ => msg.to.to_string(),
    };
    let channel = glob
        .channel_list
        .read()
        .await
        .get(&channel_name)
        .ok_or_else(|| format!("No channel named {}", msg.to))?
        .clone();
    if !channel.has_user(token).await {
        return Err(format!(
            "Tried sending message to {} before joining it",
            channel.name
        ));
    }
    let packet = send_message(token.as_ref(), channel.name(), msg.content);
    for c in channel
        .users
        .read()
        .await
        .iter()
        .filter(|t| t.id() != token.id())
    {
        c.enqueue(&packet).await;
    }
    Ok(())
}

pub async fn private(data: &[u8], token: &Arc<dyn Token>, glob: &Glob) -> Result<(), String> {
    let msg = common(data, token.as_ref(), glob).await?;
    glob.token_list
        .read()
        .await
        .values()
        .find(|t| t.username() == msg.to)
        .ok_or_else(|| format!("{} is not active!", msg.to))?
        .enqueue_vec(send_message(token.as_ref(), msg.to, msg.content))
        .await;
    Ok(())
}
