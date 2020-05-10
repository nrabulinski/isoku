use super::MatchSettings;
use crate::{
    packets::server::{
        channel_join_success, create_match, match_join_success, match_transfer_host,
    },
    Channel, Glob, Match, Token,
};
use std::sync::Arc;

pub async fn handle(data: &[u8], token: &Arc<Token>, glob: &Glob) -> Result<Vec<u8>, String> {
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
    *(token.multi.lock().await) = Some(Arc::downgrade(&m));
    let mut res = Vec::new();
    res.append(&mut match_join_success(&m).await);
    res.append(&mut match_transfer_host());
    let packet = create_match(&m).await;
    for u in glob.lobby.read().await.iter() {
        if u.id == token.id {
            continue;
        }
        u.queue.lock().await.extend_from_slice(&packet);
    }
    let ch = {
        let mut list = glob.channel_list.write().await;
        Channel::new(&mut list, &format!("#multi_{}", m.id), "", false)
    };
    if ch.user_join(token.clone()).await {
        token.join_channel(Arc::downgrade(&ch)).await;
        res.append(&mut channel_join_success(&ch));
    }
    println!("MATCH CREATED: {:?}", m);
    Ok(res)
}
