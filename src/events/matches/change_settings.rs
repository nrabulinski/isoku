use super::MatchSettings;
use crate::{bot, packets::server::update_match, r#match::SlotStatus, Glob, Token};
use std::sync::atomic::Ordering;
use tracing::{error, instrument};

#[instrument(skip(data, token, glob), target = "change_match_settings")]
pub async fn handle(data: &[u8], token: &dyn Token, glob: &Glob) -> Result<(), String> {
    let player = token.as_player().ok_or("You're a bot")?;
    let multi = {
        let mut mutex = player.multi.lock().await;
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
    if *multi.host_id.read().await != token.id() {
        return Err("You ain't even the host here".to_string());
    }
    let data =
        MatchSettings::decode(data).map_err(|_| "Couldn't decode match settings".to_string())?;

    *multi.name.write().await = data.name.to_string();
    multi.in_progress.store(data.in_progress, Ordering::SeqCst);
    *multi.beatmap_name.write().await = data.beatmap_name.to_string();
    let md5_old = {
        let mut md5 = multi.beatmap_md5.write().await;
        let old = (*md5).clone();
        *md5 = data.beatmap_md5.to_string();
        old
    };
    multi.beatmap_id.store(data.beatmap_id, Ordering::SeqCst);
    multi.game_mode.store(data.game_mode, Ordering::SeqCst);
    let mods_old = multi.mods.swap(data.mods, Ordering::SeqCst);
    if mods_old != data.mods || md5_old != data.beatmap_md5 {
        for slot in multi.slots.iter() {
            slot.status.compare_and_swap(
                SlotStatus::Ready as u8,
                SlotStatus::NotReady as u8,
                Ordering::SeqCst,
            );
        }
    }
    multi.freemod.store(data.freemod, Ordering::SeqCst);
    let packet = update_match(&multi).await;
    for slot in multi
        .slots
        .iter()
        .filter(|s| s.status.load(Ordering::SeqCst) & SlotStatus::Occupied as u8 > 0)
    {
        let t = slot.token.read().await;
        match t.as_ref() {
            Some(player) => player.enqueue(&packet).await,
            None => {
                error!("Match slots corrupted!");
                slot.status.store(SlotStatus::Free as u8, Ordering::SeqCst);
                bot::send_message(
                    "Looks like a slot in this match got corrupted, the issue should be fixed next time settings of this match are updated",
                    &format!("#multi_{}", multi.id),
                    glob).await.map_err(|e| {
                        error!("Couln't send a message to multi channel! (error: {:?})", e);
                    }).ok();
            }
        }
    }
    Ok(())
}
