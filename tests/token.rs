use isoku::osu::{token::Token, List};
use std::sync::Arc;

#[test]
fn get_username() {
    let token_list: List<Token> = List::new();
    let token = token_list.add_token(0, "nrabulinski".to_string());
    let found = token_list.get_username("nrabulinski").unwrap();
    assert!(Arc::ptr_eq(&token, &found));
}
