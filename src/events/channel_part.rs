use crate::{
    packets::{server::channel_kicked, OsuEncode},
    Glob, Token,
};
use std::sync::Arc;

pub async fn handle(data: &[u8], token: &Arc<Token>, glob: &Glob) -> Result<Vec<u8>, String> {
    let (channel_name, _) =
        str::decode(data).map_err(|_| "Couldn't parse channel name".to_string())?;
    println!("\nCHANNEL PART {:?} {}\n", token as &Token, channel_name);
    let channel = glob
        .channel_list
        .read()
        .await
        .get(channel_name)
        .ok_or_else(|| format!("Channel {} doesn't exist", channel_name))?
        .clone();
    if channel.user_part(token).await {
        Ok(channel_kicked(&channel))
    } else {
        Err(format!("Couldn't leave {}!", channel_name))
    }
}
