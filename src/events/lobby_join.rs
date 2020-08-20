use crate::{packets::server::create_match, Glob, Token};
use std::sync::Arc;

pub async fn handle(token: &Arc<dyn Token>, glob: &Glob) -> Result<(), String> {
    if glob
        .lobby
        .read()
        .await
        .iter()
        .any(|t| Arc::ptr_eq(t, token))
    {
        return Err("But you already are in the lobby??".to_string());
    }

    glob.lobby.write().await.push(token.clone());

    for m in glob.match_list.read().await.values() {
        token.enqueue_vec(create_match(m).await).await;
    }
    Ok(())
}
