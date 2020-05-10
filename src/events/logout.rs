use crate::{packets::server::logout, Glob};

pub async fn handle(token: &str, glob: &Glob) -> Result<(), &'static str> {
    let user = glob
        .token_list
        .write()
        .await
        .remove(token)
        .ok_or("No such user logged in")?;
    for c in user.channels.lock().await.iter() {
        let c = match c.upgrade() {
            Some(c) => c,
            None => continue,
        };
        c.user_part(&user).await;
    }
    let packet = logout(&user);
    for u in glob.token_list.read().await.values() {
        u.queue.lock().await.extend_from_slice(&packet);
    }
    Ok(())
}
