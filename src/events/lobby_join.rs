use crate::{packets::server::create_match, Glob, Token};
use std::sync::Arc;

pub async fn handle(token: &Arc<Token>, glob: &Glob) -> Result<Vec<u8>, String> {
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

    let mut res = Vec::new();
    for m in glob.match_list.read().await.values() {
        res.append(&mut create_match(m).await)
    }
    Ok(res)
}
