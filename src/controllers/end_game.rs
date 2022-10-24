use std::{fs::File, io::{Error, ErrorKind, Write}};

use teloxide::{Bot, types::{Message, InputFile}};

use crate::core::game_handler::RUNNING_GAMES;

pub async fn end_game(
    _: &Bot,
    message: Message,
) -> Result<InputFile, Error> {
    let chat_id = message.chat.id.to_string();
    let mut games = RUNNING_GAMES.lock().await;
    // if no game struct -> return and notify invalid state
    if !games.contains_key(&chat_id) {
        return Err(Error::new(
            ErrorKind::Other, 
            "No game currently running..".to_string())
        );
    }

    // find game struct of the chat (should always be found due to previous step)
    let game_to_play = match games.remove(&chat_id) {
        Some(game) => game,
        None => return Err(Error::new(
            ErrorKind::Other, 
            "Error finding a running game! Invalid state on game fetch".to_string())
        )
    };

    let file_name = append_file_to_path(game_to_play.generate_file_name());


    // try to end game
    let game_result = game_to_play.end_game();
    let html = match game_result {
        Ok(message) => message,
        Err(e) => return Err(Error::new(
            ErrorKind::Other, 
            format!("Error ending game: {}", e))
        )
    };
    match save_file(html, file_name.clone()) {
        Ok(_) => Ok(send_file(file_name)),
        Err(e) => Err(Error::new(
            ErrorKind::Other, 
            format!("Error saving game fle: {}", e))
        )
    }
}

fn append_file_to_path(generated_file_name: String) -> String {
    format!("./res/games/{}", generated_file_name)
        .replace(' ', "_")
        .replace('-', "_")
        .replace(':', "_")
}


fn send_file(file_name: String) -> InputFile {
    InputFile::file(file_name)
}

fn save_file(contents: String, file_name: String) -> Result<(), Error> {
    let mut file = match File::create(file_name) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    match file.write_all(contents.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}