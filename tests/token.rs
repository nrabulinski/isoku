#![feature(option_expect_none)]
use isoku::{Glob, PlayerToken};
use std::{sync::Arc, time::Duration};

async fn setup() -> Glob {
    dotenv::dotenv().ok();
    Glob::new().await
}

#[tokio::test(core_threads = 2)]
async fn timeout() {
    let glob = Arc::new(setup().await);
    let token = {
        let t = PlayerToken::new_with_timeout(
            glob.clone(),
            0,
            "nrabulinski".to_string(),
            Duration::from_secs(1),
        )
        .await;
        Arc::downgrade(&t)
    };
    tokio::time::delay_for(Duration::from_secs(2)).await;
    token.upgrade().expect_none("Token not dropped");
}
