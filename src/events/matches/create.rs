use super::MatchSettings;
use crate::{
    packets::server::{
        channel_join_success, create_match, match_join_success, match_transfer_host,
    },
    token::Token,
    Channel, Glob, Match,
};
use std::sync::Arc;

pub async fn handle(data: &[u8], token: &Arc<dyn Token>, glob: &Glob) -> Result<(), String> {
    let player = token.as_player().ok_or("You're a bot")?;
    let data =
        MatchSettings::decode(data).map_err(|_| "Couldn't parse match settings".to_string())?;

    println!("NEW MATCH! {:?}", data);
    let m = {
        let mut list = glob.match_list.write().await;
        Match::new(
            &mut list,
            data.name,
            data.password,
            data.beatmap_name,
            data.beatmap_id,
            data.beatmap_md5,
            token,
        )
        .await
    };
    *(player.multi.lock().await) = Some(Arc::downgrade(&m));
    token.enqueue_vec(match_join_success(&m).await).await;
    token.enqueue_vec(match_transfer_host()).await;
    let packet = create_match(&m).await;
    for u in glob.lobby.read().await.iter() {
        if u.id() == token.id() {
            continue;
        }
        u.enqueue(&packet).await;
    }
    let ch = {
        let mut list = glob.channel_list.write().await;
        Channel::new(&mut list, &format!("#multi_{}", m.id), "", false)
    };
    if ch.user_join(token.clone()).await {
        token.join_channel(Arc::downgrade(&ch)).await;
        token.enqueue_vec(channel_join_success(&ch)).await;
    }
    println!("MATCH CREATED: {:?}", m);
    Ok(())
}
