use crate::{
    packets::{server::user_stats, OsuEncode},
    token::Token,
    Glob,
};

pub async fn handle(data: &[u8], token: &dyn Token, glob: &Glob) -> Result<(), String> {
    let (users, _) = <[i32]>::decode(data).map_err(|_| "Couldn't decode data".to_string())?;
    let mut res = Vec::new();
    for t in glob
        .token_list
        .read()
        .await
        .values()
        .filter(|t| users.contains(&t.id()))
    {
        res.append(&mut (user_stats(t.as_ref()).await));
    }
    token.enqueue_vec(res).await;
    Ok(())
}
