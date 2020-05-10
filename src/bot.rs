use crate::{packets::server::send_message, Glob, Token};

pub async fn handle_command(cmd: &str, token: &Token, glob: &Glob) -> Result<(), String> {
    let mut cmd = cmd.split(' ');
    let command = cmd.next().unwrap();
    match command {
        "echo" => {
            token.queue.lock().await.append(&mut send_message(
                &glob.bot,
                &token.username,
                &cmd.collect::<Vec<&str>>().join(" "),
            ));
            Ok(())
        }
        _ => Err(format!("No such command \"{}\"! Try: !help", command)),
    }
}
