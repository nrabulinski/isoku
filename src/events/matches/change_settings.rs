use super::MatchSettings;
use crate::{Glob, Token};
use std::sync::atomic::Ordering;

pub async fn handle(data: &[u8], token: &Token, glob: &Glob) -> Result<(), String> {
    let multi = {
        let mut mutex = token.multi.lock().await;
        mutex
            .as_ref()
            .ok_or_else(|| "You aren't in a multiplayer lobby")?
            .clone()
            .upgrade()
            .ok_or_else(|| {
                *mutex = None;
                "The lobby doesn't exist anymore".to_string()
            })?
    };
    if *multi.host_id.read().await != token.id {
        return Err("You ain't even the host here".to_string());
    }
    let data =
        MatchSettings::decode(data).map_err(|_| "Couldn't decode match settings".to_string())?;

    *multi.name.write().await = data.name.to_string();
    multi.in_progress.store(data.in_progress, Ordering::SeqCst);
    *multi.beatmap_name.write().await = data.beatmap_name.to_string();
    *multi.beatmap_md5.write().await = data.beatmap_md5.to_string();
    multi.beatmap_id.store(data.beatmap_id, Ordering::SeqCst);
    multi.game_mode.store(data.game_mode, Ordering::SeqCst);
    let mods_old = multi.mods.swap(data.mods, Ordering::SeqCst);
    Ok(())
}
