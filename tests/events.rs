#![feature(option_expect_none)]

use isoku::{events as e, packets::OsuEncode, Channel, Glob, Token};
use std::sync::Arc;

async fn setup() -> Glob {
    dotenv::dotenv().ok();
    Glob::new().await
}

#[tokio::test]
async fn logout() {
    let glob = setup().await;
    let test_channel = {
        let mut list = glob.channel_list.write().await;
        Channel::new(&mut list, "test", "", false)
    };
    let (token_ptr, token) = {
        let mut list = glob.token_list.write().await;
        let token = Token::new(&mut list, 0, "nrabulinski".to_string());
        assert!(test_channel.user_join(token.clone()).await);
        token.join_channel(Arc::downgrade(&test_channel)).await;
        (Arc::downgrade(&token), token.token.clone())
    };
    e::logout::handle(&token, &glob).await.unwrap();
    assert_eq!(test_channel.users.read().await.len(), 0);
    token_ptr.upgrade().expect_none("Token not dropped");
}

#[tokio::test]
async fn channel_join() {
    let glob = setup().await;
    let test_channel = {
        let mut list = glob.channel_list.write().await;
        Channel::new(&mut list, "test", "", false)
    };
    let token = {
        let mut list = glob.token_list.write().await;
        let token = Token::new(&mut list, 0, "nrabulinski".to_string());
        token
    };
    let mut event_data = Vec::with_capacity(OsuEncode::encoded_size("test"));
    "test".encode(&mut event_data);
    e::channel_join::handle(&event_data, &token, &glob)
        .await
        .unwrap();
    assert!(test_channel.has_user(&token).await);
    assert_eq!(test_channel.users.read().await.len(), 1);
    let user_channels = token.channels.lock().await;
    assert_eq!(user_channels.len(), 1);
    let ch = user_channels.first().unwrap().upgrade().unwrap();
    assert_eq!(ch.name, "test");
}
