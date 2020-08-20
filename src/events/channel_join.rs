use crate::{
    packets::{server::channel_join_success, OsuEncode},
    token::Token,
    Glob,
};
use std::sync::Arc;

pub async fn handle(data: &[u8], token: &Arc<dyn Token>, glob: &Glob) -> Result<(), String> {
    let (name, _) = str::decode(data).map_err(|_| "Couldn't parse channel's name")?;
    println!("\nCHANNEL JOIN {:?} {}\n", token.as_ref(), name);
    match glob.channel_list.read().await.get(name) {
        Some(channel) => {
            if channel.user_join(token.clone()).await {
                token.join_channel(Arc::downgrade(channel)).await;
                token.enqueue_vec(channel_join_success(channel)).await;
                Ok(())
            } else {
                Err(format!("Couldn't join channel {}", name))
            }
        }
        None => Err(format!("Couldn't find channel {}", name)),
    }
}
