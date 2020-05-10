use crate::{event_data, packets::server::user_panel, token::Action, Glob, Token};
use std::convert::TryFrom;

#[event_data]
#[derive(Debug)]
struct ActionData {
    id: u8,
    text: &str,
    md5: &str,
    mods: u32,
}

pub async fn handle(data: &[u8], token: &Token, glob: &Glob) -> Result<(), String> {
    let data = ActionData::decode(data).map_err(|_| "Couldn't decode data".to_string())?;
    println!("{:?} is changing their action to {:?}", token, data);
    let action = Action::try_from(data.id).map_err(|_| format!("Unknown action id {}", data.id))?;
    // Update user's stats and drop the r/w lock
    {
        let mut s = token.stats.write().await;
        s.action = action;
        s.action_text = data.text.to_string();
        s.action_md5 = data.md5.to_string();
        s.action_mods = data.mods;
    }
    let packet = user_panel(token);
    for t in glob.token_list.read().await.values() {
        t.queue.lock().await.extend_from_slice(&packet);
    }
    Ok(())
}
