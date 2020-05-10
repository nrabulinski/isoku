use crate::{
    packets::{server::channel_join_success, OsuEncode},
    Glob, Token,
};
use std::sync::Arc;

pub async fn handle(data: &[u8], token: &Arc<Token>, glob: &Glob) -> Result<Vec<u8>, String> {
    let (name, _) = str::decode(data).map_err(|_| "Couldn't parse channel's name")?;
    println!("\nCHANNEL JOIN {:?} {}\n", token as &Token, name);
    match glob.channel_list.read().await.get(name) {
        Some(channel) => {
            if channel.user_join(token.clone()).await {
                token.join_channel(Arc::downgrade(channel)).await;
                Ok(channel_join_success(channel))
            } else {
                Err(format!("Couldn't join channel {}", name))
            }
        }
        None => Err(format!("Couldn't find channel {}", name)),
    }
}
