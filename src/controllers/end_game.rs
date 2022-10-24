use teloxide::{Bot, types::Message};

use crate::core::game_handler::RUNNING_GAMES;

pub async fn end_game(
    _: &Bot,
    message: Message,
) -> String {
    let chat_id = message.chat.id.to_string();
    let mut games = RUNNING_GAMES.lock().await;
    
    // if no game struct -> return and notify invalid state
    if !games.contains_key(&chat_id) {
        return "No game currently running..".to_string();
    }

    // find game struct of the chat (should always be found due to previous step)
    let game_to_play = match games.get_mut(&chat_id) {
        Some(game) => game,
        None => return "Error finding a running game! Invalid state on game fetch".to_string()
    };

    // try to end game
    let game_result = game_to_play.end_game();
    match game_result {
        Ok(message) => message,
        Err(e) => format!("Error ending game: {}", e.to_string()) 
    }
}