use teloxide::{Bot, types::Message};

use crate::core::game_handler::RUNNING_GAMES;

pub async fn score_round(
    _: &Bot,
    message: Message,
) -> String {
    let chat_id = message.chat.id.to_string();
    let mut games = RUNNING_GAMES.lock().await;
    
    // if no game struct -> return and notify invalid state
    if !games.contains_key(&chat_id) {
        return "No game currently running...try /newgame first.".to_string();
    }

    // find game struct of the chat (should always be found due to previous step)
    let game_to_play = match games.get_mut(&chat_id) {
        Some(game) => game,
        None => return "Error finding a running game! Invalid state on game fetch".to_string()
    };

    // try to handle message
    let game_result = game_to_play.handle_round(message);
    match game_result {
        Ok(message) => message,
        Err(e) => format!("Error handling round: {}", e) 
    }
}