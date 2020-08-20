use crate::{Glob, Token};
use std::sync::Arc;

pub async fn handle(token: &Arc<dyn Token>, glob: &Glob) -> Result<(), String> {
    let mut lobby = glob.lobby.write().await;
    let pos = lobby
        .iter()
        .position(|t| Arc::ptr_eq(t, token))
        .ok_or_else(|| "You aren't in the lobby".to_string())?;
    lobby.remove(pos);
    Ok(())
}
