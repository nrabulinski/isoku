use crate::{packets::server::user_stats, Token};

pub async fn handle(token: &Token) -> Result<Vec<u8>, String> {
    // TODO: Actually update the stats, lol
    Ok(user_stats(token).await)
}
