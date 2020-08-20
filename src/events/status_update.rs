use crate::{packets::server::user_stats, Token};

pub async fn handle(token: &dyn Token) -> Result<(), String> {
    // TODO: Actually update the stats, lol
    token.enqueue_vec(user_stats(token).await).await;
    Ok(())
}
