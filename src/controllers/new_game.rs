use teloxide::{Bot, types::Message};

use crate::{core::{game_handler::RUNNING_GAMES, traits::Game}, games::table::table::Table};

pub async fn new_game(
    _: &Bot,
    message: Message,
) -> String {
    let chat_id = message.chat.id.to_string();
    let mut games = RUNNING_GAMES.lock().await;
    
    // if no game struct -> create a game struct
    if !games.contains_key(&chat_id) {
        games.insert(chat_id.clone(), get_chat_default_game());
    }

    // find game struct of the chat (should always be created due to previous step)
    let game_to_play = match games.get_mut(&chat_id) {
        Some(game) => game,
        None => return "Error starting a game! Invalid state on game fetch".to_string()
    };

    // try to start game
    let game_result = game_to_play.start_game();
    match game_result {
        Ok(message) => message,
        Err(e) => format!("Error starting game: {}", e.to_string()) 
    }
}

fn get_chat_default_game() -> Box<dyn Game + Send> {
    Box::new(Table::new())
}