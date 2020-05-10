#![feature(option_expect_none)]

use isoku::{events as e, Glob, Token};
#[macro_use]
extern crate lazy_static;
use std::sync::Arc;

#[tokio::test]
async fn logout() {
    dotenv::dotenv().ok();
    let glob = Glob::new().await;
    let (token_ptr, token) = {
        let mut list = glob.token_list.write().await;
        let token = Token::new(&mut list, 0, "nrabulinski".to_string());
        (Arc::downgrade(&token), token.token.clone())
    };
    e::logout::handle(&token, &glob).await.unwrap();
    token_ptr.upgrade().expect_none("Token not dropped");
}
